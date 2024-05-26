use std::collections::HashMap;
use std::error::Error as StdErr;
use std::str::FromStr;

mod const_value;
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
    Address, HumanCapacity, ScriptId, SECP256K1,
};

use ckb_types::{bytes::Bytes, core::{BlockView, ScriptHashType, TransactionView}, packed::{CellOutput, Script, WitnessArgs}, prelude::*, H256};
use ckb_types::packed::{Byte32,CellDepBuilder, OutPoint, ScriptOpt};
use clap::Parser;
use const_value::const_value::CODE_HASH;
use crate::const_value::const_value::TX_HASH;

/// Transfer some CKB from one sighash address to other address
/// # Example:
///     ./target/debug/examples/transfer_from_sighash \
///       --sender-key <key-hex> \
///       --receiver <address> \
///       --capacity 61.0
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The sender private key (hex string)
    #[clap(long, value_name = "KEY", default_value = "a5808e79c243d8e026a034273ad7a5ccdcb2f982392fd0230442b1734c98a4c2")]
    sender_key: H256,

    /// The receiver address
    #[clap(long, value_name = "ADDRESS", default_value = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsq2prryvze6fhufxkgjx35psh7w70k3hz7c3mtl4d")]
    receiver: Address,

    /// The capacity to transfer (unit: CKB, example: 102.43)
    #[clap(long, value_name = "CKB", default_value = "100")]
    capacity: HumanCapacity,

    /// CKB rpc url
    #[clap(long, value_name = "URL", default_value = "http://127.0.0.1:8114")]
    ckb_rpc: String,
}

fn main() -> Result<(), Box<dyn StdErr>> {
    // Parse arguments
    let args = Args::parse();

    let sender_key = secp256k1::SecretKey::from_slice(args.sender_key.as_bytes())
        .map_err(|err| format!("invalid sender secret key: {}", err))?;
    let sender = {
        let pubkey = secp256k1::PublicKey::from_secret_key(&SECP256K1, &sender_key);
        let hash160 = blake2b_256(&pubkey.serialize()[..])[0..20].to_vec();
        Script::new_builder()
            .code_hash(SIGHASH_TYPE_HASH.pack())
            .hash_type(ScriptHashType::Type.into())
            .args(Bytes::from(hash160).pack())
            .build()
    };

    let tx = build_transfer_tx(&args, sender, sender_key)?;

    // Send transaction
    let json_tx = json_types::TransactionView::from(tx);
    // println!("tx: {}", serde_json::to_string_pretty(&json_tx).unwrap());
    let outputs_validator = Some(json_types::OutputsValidator::Passthrough);
    let _tx_hash = CkbRpcClient::new(args.ckb_rpc.as_str())
        .send_transaction(json_tx.inner, outputs_validator)
        .expect("send transaction");
    println!(">>> tx sent! <<<");

    Ok(())
}

fn build_transfer_tx(
    args: &Args,
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

    // Build:
    //   * CellDepResolver
    //   * HeaderDepResolver
    //   * CellCollector
    //   * TransactionDependencyProvider

    let tx = TX_HASH.parse::<H256>().unwrap().0;
    let tx_hash = Byte32::from_slice(&tx).unwrap();
    let code = CODE_HASH.parse::<H256>().unwrap().0;
    let code_hash = Byte32::from_slice(&code).unwrap();

    let type_out_point = OutPoint::new(tx_hash, 0);
    let ckb_client = CkbRpcClient::new(args.ckb_rpc.as_str());

    let type_script_id = ScriptId::new(code_hash.unpack(), ScriptHashType::Data2.into());

    let type_cell_dep = CellDepBuilder::default().out_point(type_out_point).build();

    let mut cell_dep_resolver = {
        let genesis_block = ckb_client.get_block_by_number(0.into())?.unwrap();
        DefaultCellDepResolver::from_genesis(&BlockView::from(genesis_block))?
    };

    cell_dep_resolver.insert(type_script_id, type_cell_dep, String::from("data_check"));
    let header_dep_resolver = DefaultHeaderDepResolver::new(args.ckb_rpc.as_str());
    let mut cell_collector = DefaultCellCollector::new(args.ckb_rpc.as_str());
    let tx_dep_provider = DefaultTransactionDependencyProvider::new(args.ckb_rpc.as_str(), 10);

    let type_script = Script::new_builder()
        .code_hash(code_hash)
        .hash_type(ScriptHashType::Data2.into())
        // .args(Bytes::from("apple").pack())
        .build();

    let type_script_opt = ScriptOpt::new_builder()
        .set(Some(type_script)).build();
    //
    // println!("{:?}", type_script_opt);

    let auto_success_script = Script::default();

    // Build the transaction
    let output_cells = vec![

        CellOutput::new_builder()
        .lock(auto_success_script.clone())
        .type_(type_script_opt.clone())
        .capacity(args.capacity.0.pack())
        .build(),
        CellOutput::new_builder()
        .lock(auto_success_script)
        .type_(type_script_opt)
        .capacity(args.capacity.0.pack())
        .build(),
    ];
    let outputs_data = vec![Bytes::from("apple"), Bytes::from("tomato")];
    let outputs = output_cells.into_iter().zip(outputs_data.into_iter()).collect();


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