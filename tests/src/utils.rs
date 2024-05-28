use std::time::Instant;

use ckb_testtool::builtin::ALWAYS_SUCCESS;
use ckb_testtool::bytes::Bytes;
use ckb_testtool::ckb_types::core::{TransactionBuilder, TransactionView};
use ckb_testtool::ckb_types::packed::{CellDep, CellInput, CellOutput};
use ckb_testtool::ckb_types::prelude::{Builder, Entity, Pack};
use ckb_testtool::context::Context;

use crate::Loader;

const MAX_CYCLES: u64 = 1_000_000_000_000;

fn build_test_context(
    proof_file: Bytes,
    contract: &str,
) -> (Context, TransactionView) {
    // deploy contract.
    let mut context = Context::default();
    let contract_bin: Bytes = Loader::default().load_binary(contract);
    let contract_out_point = context.deploy_cell(contract_bin);
    // Deploy always_success script as lock script.
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // Build LOCK script using always_success script.
    let lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("build lock script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    // Build TYPE script using the ckb-zkp contract
    let type_script = context
        .build_script(&contract_out_point, Bytes::default())
        .expect("build type script");
    let type_script_dep = CellDep::new_builder().out_point(contract_out_point).build();

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(((proof_file.len()) as u64).pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point)
        .build();
    let outputs = vec![
        CellOutput::new_builder()
            .capacity((proof_file.len() as u64).pack())
            .lock(lock_script.clone())
            .type_(Some(type_script).pack())
            .build(),
    ];

    let outputs_data = vec![proof_file];

    // build transaction
    let tx = TransactionBuilder::default()
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(lock_script_dep)
        .cell_dep(type_script_dep)
        .build();
    (context, tx)
}

pub fn proving_test(proof: Bytes, contract: &str, name: &str) {
    let (mut context, tx) = build_test_context(proof, contract);

    let tx = context.complete_tx(tx);

    let start = Instant::now();
    match context.verify_tx(&tx, MAX_CYCLES) {
        Ok(cycles) => {
            println!("{}: cycles: {}", name, cycles);
        }
        Err(err) => panic!("Failed to pass test: {}", err),
    }
    println!(
        "Verify Mini circuit use {} Time: {:?}",
        name,
        start.elapsed()
    );
}
