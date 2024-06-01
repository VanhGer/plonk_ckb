use ark_bls12_381::Fr;
use ark_serialize::*;
use kzg::srs::Srs;
use sha2::Sha256;

use plonk::prover;

use crate::utils::proving_test;

// use ckb_tool::ckb_types::{
//     bytes::Bytes,
//     core::{TransactionBuilder, TransactionView},
//     packed::*,
//     prelude::*,
// };

pub(crate) const MAX_CYCLES: u64 = 3_500_000_000;

#[test]
fn test_plonk_contract() {
    use ark_bls12_381::Fr;
    use plonk::parser::Parser;

    let mut parser = Parser::default();
    parser.add_witness("x", Fr::from(3));
    parser.add_witness("y", Fr::from(2));
    parser.add_witness("z", Fr::from(5));

    // generate proof
    let compiled_circuit = parser.parse("x + y + z*z = 30").compile().unwrap();
    let srs = Srs::new(compiled_circuit.size);
    let proof = prover::generate_proof::<Sha256>(&compiled_circuit, srs);

    let mut proof_bytes = Vec::new();
    proof.serialize_compressed(&mut proof_bytes).unwrap();
    proving_test(
        proof_bytes.into(),
        "plonk_verifier",
        "plonk_verifier verify",
    );
}
