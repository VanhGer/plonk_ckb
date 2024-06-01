use std::collections::HashMap;
use std::error::Error as StdErr;
use std::fs;
use std::io::BufReader;
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
    traits::{
        DefaultCellCollector, DefaultCellDepResolver, DefaultHeaderDepResolver,
        DefaultTransactionDependencyProvider, SecpCkbRawKeySigner,
    },
    tx_builder::{transfer::CapacityTransferBuilder, CapacityBalancer, TxBuilder},
    unlock::{ScriptUnlocker, SecpSighashUnlocker},
    ScriptId, SECP256K1,
};
use ckb_types::packed::{Byte32, CellDepBuilder, OutPoint};
use ckb_types::{
    bytes::Bytes,
    core::{BlockView, ScriptHashType, TransactionView},
    packed::{CellOutput, Script, WitnessArgs},
    prelude::*,
    H256,
};
use clap::{Parser, Subcommand};
use config::ConfigError::Message;
use config::{Config, ConfigError, Environment, File};
use glob::glob;
use serde::Deserialize;
use sha2::Sha256;

use kzg::srs::Srs;
use plonk::prover;

pub const TO_SHANNON: u64 = 100000000;

/// This is a tool for generating proof and send it to CKB network to verify.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Configuration file paths
    #[arg(short, long, default_value = "config/00-default.toml")]
    config_path: Vec<String>,
}

/// Enumeration of possible commands for the application
#[derive(Subcommand, Clone, Debug)]
enum Commands {
    /// Print the configuration
    Config,
}

/// Entry point for the application
fn main() -> Result<(), Box<dyn StdErr>> {
    // Parse command-line arguments
    let args = Args::parse();
    // Parse configuration options from config files
    let options: Options = parse_options(args.config_path)?;

    // Handle the 'config' command
    if let Some(Commands::Config) = args.command {
        println!("{:#?}", options);
        return Ok(());
    }

    // Initialize the sender's private key
    let sender_key: H256 = H256::from_str(&options.sender_key)?;
    let sender_key = secp256k1::SecretKey::from_slice(sender_key.as_bytes())
        .map_err(|err| format!("Invalid sender secret key: {}", err))?;

    // Create the sender's lock script
    let sender = {
        let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &sender_key);
        let hash160 = blake2b_256(pubkey.serialize())[0..20].to_vec();
        Script::new_builder()
            .code_hash(SIGHASH_TYPE_HASH.pack())
            .hash_type(ScriptHashType::Type.into())
            .args(Bytes::from(hash160).pack())
            .build()
    };

    // Build the PLONK verifier transaction
    let tx = build_plonk_verifier_tx(&options, sender, sender_key)?;

    // Send the transaction to the CKB network
    let json_tx = json_types::TransactionView::from(tx);
    let outputs_validator = Some(json_types::OutputsValidator::Passthrough);
    let rpc_client = CkbRpcClient::new(&options.ckb_rpc);
    let tx_hash = rpc_client
        .send_transaction(json_tx.inner, outputs_validator)
        .expect("Failed to send transaction");

    // Monitor the transaction status
    let mut count = 0;
    let mut last_state = String::new();
    loop {
        sleep(Duration::from_secs(1));
        match rpc_client
            .get_pool_tx_detail_info(tx_hash.clone())
            .unwrap()
            .entry_status
            .as_str()
        {
            "unknown" => {
                if last_state == "proposed" {
                    break;
                }
            }
            str => last_state = str.to_string(),
        }
        count += 1;
        if count > 20 {
            println!("Transaction timeout!");
            break;
        }
    }

    println!("Transaction took: {} secs", count);
    let tx_info = rpc_client
        .get_transaction(tx_hash.clone())
        .expect("Transaction retrieval failed");
    eprintln!("tx_hash = {:#?}", tx_hash.to_string());
    eprintln!("tx_info = {:#?}", tx_info);
    Ok(())
}

/// Builds a transaction for the PLONK verifier using the provided options and sender information
///
/// # Arguments
///
/// * `options` - Configuration options for the application
/// * `sender_lock_script` - The sender lock script
/// * `sender_key` - The secret key of the sender
///
/// # Returns
///
/// A Result containing the transaction view or an error
fn build_plonk_verifier_tx(
    options: &Options,
    sender_lock_script: Script,
    sender_key: secp256k1::SecretKey,
) -> Result<TransactionView, Box<dyn StdErr>> {
    // Initialize the signer and unlocker
    let signer = SecpCkbRawKeySigner::new_with_secret_keys(vec![sender_key]);
    let sighash_unlocker = SecpSighashUnlocker::from(Box::new(signer) as Box<_>);
    let sighash_script_id = ScriptId::new_type(SIGHASH_TYPE_HASH.clone());
    let mut unlockers = HashMap::default();
    unlockers.insert(
        sighash_script_id,
        Box::new(sighash_unlocker) as Box<dyn ScriptUnlocker>,
    );

    // Initialize the capacity balancer with a placeholder witness
    let placeholder_witness = WitnessArgs::new_builder()
        .lock(Some(Bytes::from(vec![0u8; 65])).pack())
        .build();
    let balancer = CapacityBalancer::new_simple(sender_lock_script, placeholder_witness, 1000);

    // Parse transaction and code hashes
    let tx = options.tx_hash[2..].parse::<H256>().unwrap().0;
    let tx_hash = Byte32::from_slice(&tx).unwrap();
    let code = options.verifier_code_hash[2..].parse::<H256>().unwrap().0;
    let code_hash = Byte32::from_slice(&code).unwrap();

    // Create type out point and script ID
    let type_out_point = OutPoint::new(tx_hash, 0);
    let type_script_id = ScriptId::new(code_hash.unpack(), ScriptHashType::Data2);

    // Initialize RPC client
    let ckb_client = CkbRpcClient::new(&options.ckb_rpc);

    // Resolve cell dependencies from genesis block
    let mut cell_dep_resolver = {
        let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
        DefaultCellDepResolver::from_genesis(&BlockView::from(genesis_block))?
    };
    cell_dep_resolver.insert(type_script_id, CellDepBuilder::default().out_point(type_out_point).build(), "plonk_check".to_string());

    // Initialize other resolvers and collectors
    let header_dep_resolver = DefaultHeaderDepResolver::new(&options.ckb_rpc);
    let mut cell_collector = DefaultCellCollector::new(&options.ckb_rpc);
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(&options.ckb_rpc, 0);

    // Build the type script
    let type_script = Script::new_builder()
        .code_hash(code_hash)
        .hash_type(ScriptHashType::Data2.into())
        .build();

    // Placeholder script for auto-success
    let auto_success_script = Script::default();

    // Generate the PLONK proof
    let proof_bytes = generate_plonk(options);

    // Compute the initial and cell capacities
    let init_cap = type_script.occupied_capacity().unwrap().as_u64()
        + auto_success_script.occupied_capacity().unwrap().as_u64();
    let cell_cap = init_cap + (proof_bytes.len() as u64) * TO_SHANNON;

    // Create the output cells and data
    let output_cells = vec![CellOutput::new_builder()
        .capacity((cell_cap + (cell_cap.to_be_bytes().len() as u64) * TO_SHANNON).pack())
        .lock(auto_success_script.clone())
        .type_(Some(type_script.clone()).pack())
        .build()];
    let outputs_data = vec![proof_bytes];
    let outputs: Vec<_> = output_cells.into_iter().zip(outputs_data).collect();

    // Build the transaction
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

/// Generates a PLONK proof using the provided options
///
/// # Arguments
///
/// * `options` - Configuration options for the application
///
/// # Returns
///
/// A byte array containing the generated PLONK proof
pub fn generate_plonk(options: &Options) -> Bytes {
    let mut parser = plonk::parser::Parser::default();
    options.witnesses.split(';').for_each(|key_value| {
        let key_value: Vec<&str> = key_value.split('=').map(|s| s.trim()).collect();
        assert_eq!(key_value.len(), 2);
        parser.add_witness(key_value[0], Fr::from(key_value[1].parse::<i32>().unwrap()));
    });

    // Generate the proof
    let compiled_circuit = parser.parse(&options.equation).compile().unwrap();

    let f = fs::File::open(&options.srs_path).expect("No file found");
    let mut reader = BufReader::new(f);
    let srs = Srs::deserialize_uncompressed_unchecked(&mut reader).expect("Should work!");

    let proof = prover::generate_proof::<Sha256>(&compiled_circuit, srs);
    let mut proof_bytes = Vec::new();
    proof.serialize_uncompressed(&mut proof_bytes).unwrap();

    proof_bytes.into()
}

/// Configuration options for the application
#[derive(Deserialize, Debug)]
pub struct Options {
    #[serde(default = "default_sender_key")]
    sender_key: String,
    #[serde(default = "default_ckb_rpc")]
    ckb_rpc: String,
    pub verifier_code_hash: String,
    pub tx_hash: String,
    pub equation: String,
    pub witnesses: String,
    pub srs_path: String,
}

/// Returns the default CKB RPC URL
fn default_ckb_rpc() -> String {
    "http://127.0.0.1:8114".to_string()
}

/// Returns the default sender private key
fn default_sender_key() -> String {
    "ace08599f3174f4376ae51fdc30950d4f2d731440382bb0aa1b6b0bd3a9728cd".to_string()
}

/// Parses the configuration options from the provided config paths
///
/// # Arguments
///
/// * `config_paths` - A vector of paths to configuration files
///
/// # Returns
///
/// A Result containing the parsed configuration options or an error
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
