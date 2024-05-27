use std::fs;
use std::io::Write;

use ark_bls12_381::Fr;
use ark_ff::One;
use ark_poly::univariate::DensePolynomial;
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};

use crate::common_processed_input_const::COMMON_PROCESSED_INPUT;
use crate::compiled_circuit::CompiledCircuit;

#[derive(CanonicalSerialize, CanonicalDeserialize)]
struct CommonProcessedInput {
    n: usize,
    k1: Fr,
    k2: Fr,
    q_lx: DensePolynomial<Fr>,
    q_rx: DensePolynomial<Fr>,
    q_mx: DensePolynomial<Fr>,
    q_ox: DensePolynomial<Fr>,
    q_cx: DensePolynomial<Fr>,
    s_sigma_1: DensePolynomial<Fr>,
    s_sigma_2: DensePolynomial<Fr>,
    s_sigma_3: DensePolynomial<Fr>,
}

impl CommonProcessedInput {
    pub fn new(compiled_circuit: CompiledCircuit) -> Self {
        let copy_constraint = compiled_circuit.copy_constraints();
        let gate_constraint = compiled_circuit.gate_constraints();
        Self {
            n: compiled_circuit.size,
            k1: copy_constraint.k1().clone(),
            k2: copy_constraint.k2().clone(),
            q_lx: gate_constraint.q_lx().clone(),
            q_rx: gate_constraint.q_rx().clone(),
            q_mx: gate_constraint.q_mx().clone(),
            q_ox: gate_constraint.q_ox().clone(),
            q_cx: gate_constraint.q_cx().clone(),
            s_sigma_1: copy_constraint.get_s_sigma_1().clone(),
            s_sigma_2: copy_constraint.get_s_sigma_2().clone(),
            s_sigma_3: copy_constraint.get_s_sigma_3().clone(),
        }
    }

    /// Create a new file contains common processed output
    /// This method shouldn't panic if used correctly
    pub fn parse(&self) {
        let mut bytes = Vec::new();

        self.serialize_uncompressed(&mut bytes).unwrap();
        let str = format!(
            "pub const COMMON_PROCESSED_INPUT:[u8;{}] = {:?};",
            bytes.len(),
            &bytes
        );
        fs::write("src/common_processed_input_const.rs", str).expect("write failed");

        println!(
            "common processed input: {:?}",
            Vec::<u8>::from(COMMON_PROCESSED_INPUT)
        );
    }
}

#[cfg(test)]
mod test {
    use ark_bls12_381::Fr;

    use crate::common_processed_input::CommonProcessedInput;
    use crate::parser::Parser;

    #[test]
    fn test() {
        let mut parser = Parser::default();
        parser.add_witness("x", Fr::from(1));
        parser.add_witness("y", Fr::from(2));
        parser.add_witness("z", Fr::from(3));
        let circuit = parser.parse("x*y+3*x^2+x*y*z=11");

        let cpo = CommonProcessedInput::new(circuit.compile().unwrap());
        cpo.parse();
    }
}
