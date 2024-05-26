use ckb_testtool::{builtin::ALWAYS_SUCCESS};
// use ckb_tool::ckb_types::{
//     bytes::Bytes,
//     core::{TransactionBuilder, TransactionView},
//     packed::*,
//     prelude::*,
// };

use ckb_testtool::{
    ckb_types::{
        bytes::Bytes,
        core::{TransactionBuilder},
    },
    context::Context,
};


use ark_bls12_381::{ Fr};
use ark_ff::{One, Zero};
use ark_serialize::*;
use ark_std::test_rng;
use ckb_testtool::ckb_types::packed::{CellDep, CellInput, CellOutput};
use ckb_testtool::ckb_types::prelude::{Builder, Entity, Pack};
use crate::utils::proving_test;

use super::*;

pub(crate) const MAX_CYCLES: u64 = 1_000_000_000_000;

#[test]
fn test_sum_check() {
    // deploy contract
    let mut context = Context::default();
    let loader = Loader::default();
    let carrot_bin = loader.load_binary("data_check");
    let carrot_out_point = context.deploy_cell(carrot_bin);
    let carrot_cell_dep = CellDep::new_builder()
        .out_point(carrot_out_point.clone())
        .build();

    // prepare scripts
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let lock_script = context
        .build_script(&always_success_out_point.clone(), Default::default())
        .expect("script");
    let lock_script_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();

    // prepare cell deps
    let cell_deps: Vec<CellDep> = vec![lock_script_dep, carrot_cell_dep];

    // prepare cells
    let input_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(1000u64.pack())
            .lock(lock_script.clone())
            .build(),
        Bytes::new(),
    );
    let input = CellInput::new_builder()
        .previous_output(input_out_point.clone())
        .build();

    let type_script = context
        .build_script(&carrot_out_point, Bytes::new())
        .expect("script");

    let outputs = vec![
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script.clone())
            .type_(Some(type_script.clone()).pack())
            .build(),
        CellOutput::new_builder()
            .capacity(500u64.pack())
            .lock(lock_script)
            .build(),
    ];

    // prepare output cell data
    let outputs_data = vec![Bytes::from("apple"), Bytes::from("tomato")];
    let mut d1 = outputs_data[0].clone();
    let mut d2=  d1.pack();
    println!("d1 {:?}", d1);
    println!("d2 {:?}", d2);
    // let str =
    // build transaction
    let tx = TransactionBuilder::default()
        .cell_deps(cell_deps)
        .input(input)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .build();

    let tx = tx.as_advanced_builder().build();

    // run
    let cycles = context
        .verify_tx(&tx, MAX_CYCLES)
        .expect("pass verification");
    println!("consume cycles: {}", cycles);

    // assert_eq!(1, 2);
}






// #[test]
fn test_plonk() {
    use ark_bls12_381::{Bls12_381, Fr};
    use ark_ff::{One, Zero};
    use ark_poly::univariate::DensePolynomial;
    use ark_poly_commit::marlin_pc::MarlinKZG10;
    use blake2::Blake2s;
    use plonk_ckb::{Composer, Plonk};

    type PC = MarlinKZG10<Bls12_381, DensePolynomial<Fr>>;
    type PlonkInst = Plonk<Fr, Blake2s, PC>;

    fn ks() -> [Fr; 4] {
        [
            Fr::zero(),
            Fr::from(8_u64),
            Fr::from(13_u64),
            Fr::from(17_u64),
        ]
    }

    fn circuit() -> Composer<Fr>
    {
        let mut cs = Composer::new();
        let one = Fr::one();
        let two = one + one;
        let three = two + one;
        let four = two + two;
        let six = two + four;
        let var_one = cs.alloc_and_assign(one);
        let var_two = cs.alloc_and_assign(two);
        let var_three = cs.alloc_and_assign(three);
        let var_four = cs.alloc_and_assign(four);
        let var_six = cs.alloc_and_assign(six);
        cs.create_add_gate(
            (var_one, one),
            (var_two, one),
            var_three,
            None,
            Fr::zero(),
            Fr::zero(),
        );
        cs.create_add_gate(
            (var_one, one),
            (var_three, one),
            var_four,
            None,
            Fr::zero(),
            Fr::zero(),
        );
        cs.create_mul_gate(
            var_two,
            var_two,
            var_four,
            None,
            Fr::one(),
            Fr::zero(),
            Fr::zero(),
        );
        cs.create_mul_gate(var_one, var_two, var_six, None, two, two, Fr::zero());
        cs.constrain_to_constant(var_six, six, Fr::zero());

        cs
    }

    let rng = &mut test_rng();

    // compose
    let cs = circuit();
    let ks = ks();
    println!("Plonk: size of the circuit: {}", cs.size());

    println!("Plonk: setting up srs...");
    let srs = PlonkInst::setup(8, rng).unwrap();

    println!("Plonk: generating keys...");
    let (pk, vk) = PlonkInst::keygen(&srs, &cs, ks).unwrap();
    let mut vk_bytes = Vec::new();
    // println!("before serialize: {:?}", vk);
    vk.serialize_unchecked(&mut vk_bytes).unwrap();

    // println!("Plonk: VerifyKey length: {}", vk_bytes.len());

    // let new_vk = plonk_ckb::VerifierKey::<Fr, PC>::deserialize_unchecked(&vk_bytes[..]).unwrap();
    //assert_eq!(vk, new_vk);
    // println!("after serialize: {:?}", new_vk);
    println!("Plonk: proving...");
    let proof = PlonkInst::prove(&pk, &cs, rng).unwrap();
    let mut proof_bytes = Vec::new();
    proof.serialize_unchecked(&mut proof_bytes).unwrap();

    let new_proof = plonk_ckb::Proof::<Fr, PC>::deserialize_unchecked(&proof_bytes[..]).unwrap();
    //assert_eq!(proof, new_proof);

    println!("Plonk: proof length: {}", proof_bytes.len());

    println!("{:?}", PlonkInst::verify(&vk, cs.public_inputs(), proof));

    let mut public_bytes = Vec::new();
    cs.public_inputs()
        .to_vec()
        .serialize_unchecked(&mut public_bytes)
        .unwrap();

    //let mut new_publics = Vec::new();
    let new_publics = Vec::<Fr>::deserialize_unchecked(&public_bytes[..]).unwrap();
    assert_eq!(cs.public_inputs(), new_publics);

    println!("{:?}", PlonkInst::verify(&vk, &new_publics, new_proof));

    println!("Plonk: verifying on CKB...");

    proving_test(
        vk_bytes.into(),
        proof_bytes.into(),
        public_bytes.into(),
        "plonk_verifier",
        "plonk_verifier verify",
    );

    assert_eq!(1, 2);
}

