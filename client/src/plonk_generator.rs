use ark_bls12_381::Fr;
use ark_serialize::CanonicalSerialize;
use ckb_types::bytes::Bytes;
use sha2::Sha256;

use plonk::parser::Parser;
use plonk::prover;

pub fn generate_plonk() -> Bytes {
    let mut parser = Parser::default();
    parser.add_witness("x", Fr::from(3));

    // generate proof
    let compiled_circuit = parser.parse("x^3 + x + 5 = 35").compile().unwrap();
    let proof = prover::generate_proof::<Sha256>(&compiled_circuit);

    let mut proof_bytes = Vec::new();

    proof.serialize_compressed(&mut proof_bytes).unwrap();

    proof_bytes.into()
}
