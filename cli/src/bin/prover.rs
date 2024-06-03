use config::ConfigError::Message;
use config::{Config, ConfigError, Environment, File};
use glob::glob;
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error as StdErr;
use std::fs;
use std::io::{BufReader, Read};
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;
use ark_bls12_381::Fr;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use ckb_hash::blake2b_256;
use ckb_jsonrpc_types as json_types;
use ckb_sdk::{
    constants::SIGHASH_TYPE_HASH,
    rpc::CkbRpcClient,
    ScriptId,
    SECP256K1, traits::{
        DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
        DefaultTransactionDependencyProvider, SecpCkbRawKeySigner,
    }, tx_builder::{CapacityBalancer, transfer::CapacityTransferBuilder, TxBuilder}, unlock::{ScriptUnlocker, SecpSighashUnlocker},
};
use ckb_types::{
    bytes::Bytes,
    core::{BlockView, ScriptHashType, TransactionView},
    H256,
    packed::{CellOutput, Script, WitnessArgs},
    prelude::*,
};
use ckb_types::packed::{Byte32, CellDepBuilder, OutPoint};
use clap::{Parser, Subcommand};
use sha2::Sha256;
use kzg::srs::Srs;
use plonk::prover;

pub const TO_SHANNON: u64 = 100000000;

/// Prove something
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Config file
    #[arg(short, long, default_value = "config/00-default.toml")]
    config_path: Vec<String>,
}

#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// Print config
    Config,
}

fn main() -> Result<(), Box<dyn StdErr>> {
    // Parse arguments
    let args = Args::parse();

    let options: Options = parse_options(args.config_path)?;

    if let Some(Commands::Config) = args.command {
        println!("{:#?}", options);
        return Ok(());
    }
    let sender_key: H256 = H256::from_str(&options.sender_key)?;
    let sender_key = secp256k1::SecretKey::from_slice(sender_key.as_bytes())
        .map_err(|err| format!("invalid sender secret key: {}", err))?;
    let sender = {
        let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &sender_key);
        let hash160 = blake2b_256(&pubkey.serialize())[0..20].to_vec();
        Script::new_builder()
            .code_hash(SIGHASH_TYPE_HASH.pack())
            .hash_type(ScriptHashType::Type.into())
            .args(Bytes::from(hash160).pack())
            .build()
    };

    let tx = build_plonk_verifier_tx(&options, sender, sender_key)?;

    // Send transaction
    let json_tx = json_types::TransactionView::from(tx);
    let outputs_validator = Some(json_types::OutputsValidator::Passthrough);
    let rpc_client = CkbRpcClient::new(options.ckb_rpc.as_str());
    let tx_hash = rpc_client
        .send_transaction(json_tx.inner, outputs_validator)
        .expect("send transaction");
    let mut count = 0;
    let mut last_state = String::new();
    loop {
        sleep(Duration::from_secs(1));
        match rpc_client.get_pool_tx_detail_info(tx_hash.clone()).unwrap().entry_status.as_str() {
            "unknown" => {
                if last_state == "proposed" {
                    break;
                }
            }
            str => last_state = String::from(str)
        }
        count += 1;
    }
    println!("transaction takes: {} secs", count);
    let cur = CkbRpcClient::new(options.ckb_rpc.as_str()).get_transaction(tx_hash.clone()).expect("Tx failed");
    println!(">>> tx sent! <<<");
    println!("{:?}", tx_hash);
    println!("tx is: {:?}", cur);
    Ok(())
}

fn build_plonk_verifier_tx(
    options: &Options,
    sender: Script,
    sender_key: secp256k1::SecretKey,
) -> Result<TransactionView, Box<dyn StdErr>> {
    // Build ScriptUnlocker
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![sender_key]);
    let sighash_unlocker = SecpSighashUnlocker::from(Box::new(signer) as Box<_>);
    let sighash_script_id = ScriptId::new_type(SIGHASH_TYPE_HASH.clone());
    let mut unlockers = HashMap::default();
    unlockers.insert(
        sighash_script_id,
        Box::new(sighash_unlocker) as Box<dyn ScriptUnlocker>,
    );

    // Build CapacityBalancer
    let placeholder_witness = WitnessArgs::new_builder()
        .lock(Some(Bytes::from(vec![0u8; 65])).pack())
        .build();
    let balancer = CapacityBalancer::new_simple(sender, placeholder_witness, 1000);

    let tx = options.tx_hash[2..].parse::<H256>().unwrap().0;
    let tx_hash = Byte32::from_slice(&tx).unwrap();
    let code = options.verifier_code_hash[2..].parse::<H256>().unwrap().0;
    let code_hash = Byte32::from_slice(&code).unwrap();

    let type_out_point = OutPoint::new(tx_hash, 0);
    let ckb_client = CkbRpcClient::new(options.ckb_rpc.as_str());

    let type_script_id = ScriptId::new(code_hash.unpack(), ScriptHashType::Data2.into());

    let type_cell_dep = CellDepBuilder::default().out_point(type_out_point).build();

    let mut cell_dep_resolver = {
        let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
        DefaultCellDepResolver::from_genesis(&BlockView::from(genesis_block))?
    };

    cell_dep_resolver.insert(type_script_id, type_cell_dep, String::from("plonk_check"));
    let header_dep_resolver = DefaultHeaderDepResolver::new(options.ckb_rpc.as_str());
    let mut cell_collector = DefaultCellCollector::new(options.ckb_rpc.as_str());
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(options.ckb_rpc.as_str(), 0);

    let type_script = Script::new_builder()
        .code_hash(code_hash)
        .hash_type(ScriptHashType::Data2.into())
        .build();

    let auto_success_script = Script::default();

    // Build the transaction
    let proof_bytes = generate_plonk(options);

    // let mut fake_proof = proof_bytes.clone().to_vec();
    // fake_proof[0] = 13;
    // let fake_proof_bytes: Bytes = fake_proof.into();

    // compute cap = type_script.size + lock_script.size
    let init_cap = type_script.occupied_capacity().unwrap().as_u64()
        + auto_success_script.occupied_capacity().unwrap().as_u64();


    /// compute cap = type_script.size + lock_script.size + data.size
    let cell_cap = init_cap + (proof_bytes.len() as u64) * TO_SHANNON;

    /// cap = type_script.size + lock_script.size + data.size + cap.size
    /// send public and proof
    let output_cells = vec![
        CellOutput::new_builder()
            .capacity((cell_cap + (cell_cap.to_be_bytes().len() as u64) * TO_SHANNON).pack())
            .lock(auto_success_script.clone())
            .type_(Some(type_script.clone()).pack())
            .build(),
    ];

    let outputs_data = vec![proof_bytes];
    let outputs: Vec<_> = output_cells
        .into_iter()
        .zip(outputs_data.into_iter())
        .collect();

    let builder = CapacityTransferBuilder::new(outputs);
    let (tx, still_locked_groups) = builder.build_unlocked(
        &mut cell_collector,
        &cell_dep_resolver,
        &header_dep_resolver,
        &tx_dep_provider,
        &balancer,
        &unlockers,
    )?;
    assert!(still_locked_groups.is_empty());

    Ok(tx)
}

pub fn generate_plonk(options: &Options) -> Bytes {
    let mut parser = plonk::parser::Parser::default();
    options.witnesses.split(";").for_each(|key_value| {
        let key_value = key_value.split("=").map(|s| s.trim())
            .collect::<Vec<&str>>();
        assert_eq!(key_value.len(), 2);
        parser.add_witness(key_value[0], Fr::from(key_value[1].parse::<i32>().unwrap()));
    });

    // generate proof
    let compiled_circuit = parser.parse(&options.equation).compile().unwrap();

    let f = fs::File::open(&options.srs_path).expect("no file found");
    let mut reader = BufReader::new(f);
    let srs = Srs::deserialize_uncompressed_unchecked(&mut reader).expect("should work!");

    let proof = prover::generate_proof::<Sha256>(&compiled_circuit, srs);
    let mut proof_bytes = Vec::new();

    proof.serialize_uncompressed(&mut proof_bytes).unwrap();

    proof_bytes.into()
}

/// Configuration options for the application.
///
/// This struct represents the configuration options for the application, including server settings,
/// database configuration, endpoint for the exporter, service name, and logging configuration.
#[derive(Deserialize, Debug)]
pub struct Options {
    /// The sender private key (hex string)
    #[serde(default = "default_sender_key")]
    sender_key: String,

    /// The receiver address
    #[serde(
        default = "default_receiver"
    )]
    receiver: String,

    /// CKB rpc url
    #[serde(default = "default_ckb_rpc")]
    ckb_rpc: String,

    /// verifier_code_hash.
    pub verifier_code_hash: String,
    /// tx_hash.
    pub tx_hash: String,
    /// equation.
    pub equation: String,
    /// witnesses.
    pub witnesses: String,
    /// srs_path.
    pub srs_path: String,
}

fn default_ckb_rpc() -> String {
    "http://127.0.0.1:8114".to_string()
}

fn default_receiver() -> String {
    "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqvm52pxjfczywarv63fmjtyqxgs2syfffq2348ad".to_string()
}

fn default_sender_key() -> String {
    "ace08599f3174f4376ae51fdc30950d4f2d731440382bb0aa1b6b0bd3a9728cd".to_string()
}

pub fn parse_options<'de, T: Deserialize<'de>>(
    config_paths: Vec<String>,
) -> Result<T, ConfigError> {
    let mut config = Config::builder();

    for path in &config_paths {
        let paths = glob(path).map_err(|e| Message(e.to_string()))?;
        for entry in paths {
            let entry = entry.map_err(|e| Message(e.to_string()))?;
            config = config.add_source(File::from(entry));
        }
    }

    let config = config
        .add_source(
            Environment::default()
                .separator("__")
                .list_separator(",")
                .ignore_empty(true),
        )
        .build()?;

    config.try_deserialize()
}
