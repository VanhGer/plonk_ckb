use ark_bls12_381::{Bls12_381, Fr};
use ark_serialize::CanonicalSerialize;
use ckb_types::bytes::Bytes;
use plonk::circuit::Circuit;
use plonk::prover;
use sha2::Sha256;

pub fn generate_plonk() -> (Bytes, Bytes) {
    let compile_circuit = Circuit::default()
        .add_multiplication_gate(
            (0, 1, Fr::from(1)),
            (1, 0, Fr::from(2)),
            (0, 3, Fr::from(2)),
            Fr::from(0),
        )
        .add_multiplication_gate(
            (1, 1, Fr::from(1)),
            (0, 0, Fr::from(1)),
            (0, 2, Fr::from(1)),
            Fr::from(0),
        )
        .add_multiplication_gate(
            (2, 1, Fr::from(1)),
            (2, 6, Fr::from(3)),
            (1, 3, Fr::from(3)),
            Fr::from(0),
        )
        .add_addition_gate(
            (0, 4, Fr::from(2)),
            (2, 2, Fr::from(3)),
            (0, 5, Fr::from(5)),
            Fr::from(0),
        )
        .add_multiplication_gate(
            (2, 0, Fr::from(2)),
            (1, 4, Fr::from(3)),
            (1, 5, Fr::from(6)),
            Fr::from(0),
        )
        .add_addition_gate(
            (2, 3, Fr::from(5)),
            (2, 4, Fr::from(6)),
            (2, 5, Fr::from(11)),
            Fr::from(0),
        )
        .add_constant_gate(
            (0, 6, Fr::from(3)),
            (1, 6, Fr::from(0)),
            (1, 2, Fr::from(3)),
            Fr::from(0),
        )
        .compile()
        .unwrap();

    // generate proof
    let proof = prover::generate_proof::<Sha256>(&compile_circuit);

    let mut public_bytes = Vec::new();
    let mut proof_bytes = Vec::new();

    compile_circuit
        .serialize_uncompressed(&mut public_bytes)
        .unwrap();
    proof.serialize_uncompressed(&mut proof_bytes).unwrap();

    (public_bytes.into(), proof_bytes.into())
}
