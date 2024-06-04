use ark_bls12_381::Fr;
use ark_serialize::*;
use ckb_testtool::ckb_types::prelude::{Builder, Entity};
use sha2::Sha256;
use kzg::srs::Srs;

use plonk::prover;
use crate::utils::proving_test;

// use ckb_tool::ckb_types::{
//     bytes::Bytes,
//     core::{TransactionBuilder, TransactionView},
//     packed::*,
//     prelude::*,
// };

pub(crate) const MAX_CYCLES: u64 = 3_500_000_000;
const SRS: &[u8] = include_bytes!("../srs.bin");

#[test]
fn test_plonk_contract() {
    use ark_bls12_381::Fr;
    use plonk::parser::Parser;

    let mut parser = Parser::default();
    parser.add_witness("x", Fr::from(2));
    parser.add_witness("y", Fr::from(4));

    let srs = Srs::deserialize_uncompressed_unchecked(SRS).unwrap();

    // generate proof
    let compiled_circuit = parser.parse("x * y + x = 10").compile().unwrap();
    let proof = prover::generate_proof::<Sha256>(&compiled_circuit, srs);

    let mut proof_bytes = Vec::new();
    proof.serialize_uncompressed(&mut proof_bytes).unwrap();
    proving_test(
        proof_bytes.into(),
        "abc",
        "plonk_verifier verify",
    );

    // assert_eq!(1, 2);
}
