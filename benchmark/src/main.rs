use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ark_bls12_381::Fr;
use ark_serialize::*;
use ckb_testtool::ckb_types::core::Cycle;
use serde::Serialize;
use serde_json::to_string;
use sha2::Sha256;
use tempfile::tempdir;

use benchmark::utils::contract_test;
use kzg::srs::Srs;
use plonk::parser::Parser;
use plonk::prover;

#[derive(Serialize)]
struct Statistics {
    min: f64,
    max: f64,
    average: f64,
    f95: f64,
    f99: f64,
}

#[derive(Serialize)]
struct MetricResult {
    srs_size: i32,
    circuit_size: i32,
    statistics: Statistics,
}

#[derive(Serialize)]
struct Metrics {
    binary_size: Vec<MetricResult>,
    cycle_count: Vec<MetricResult>,
}

// Change the srs_size, circuit_size, iterations to meet your needs
fn main() {
    setup();

    // Create a temporary directory for the verifier
    let dir = tempdir().unwrap();
    let dir_path = dir.path();
    let crate_name = "my_temp_verifier";
    let crate_path = dir_path.join(crate_name);
    fs::create_dir_all(&crate_path).unwrap();

    let srs_out_path = crate_path.join("srs.bin");
    let verifier_contract_path = crate_path.join("verifier_contracts");

    // ./target/riscv64imac-unknown-none-elf/release/"$CRATE"
    let verifier_contract_built_path = verifier_contract_path.join("target").join("riscv64imac-unknown-none-elf").join("release").join(crate_name);

    let iterations: u64 = 1;
    let f95_index = (iterations as f64 * 0.95).ceil() as usize - 1;
    let f99_index = (iterations as f64 * 0.99).ceil() as usize - 1;

    let mut metrics = Metrics {
        binary_size: vec![],
        cycle_count: vec![],
    };

    // Generate SRS and run tests for different sizes
    for srs_size in 256..257 {
        generate_srs(dir_path, srs_size, &srs_out_path);

        for circuit_size in 200..201 {
            if circuit_size > srs_size { break; }

            let mut size_results = vec![];
            let mut cycle_results = vec![];

            for _ in 0..iterations {
                let (cycle, size) = test_contract_with_params(
                    dir_path,
                    crate_name,
                    &srs_out_path,
                    &verifier_contract_path,
                    &verifier_contract_built_path,
                    circuit_size,
                );
                size_results.push(size);
                cycle_results.push(cycle);
            }

            size_results.sort_unstable();
            cycle_results.sort_unstable();

            let statistics = calculate_statistics(&size_results, &cycle_results, iterations, f95_index, f99_index);
            metrics.binary_size.push(MetricResult { srs_size, circuit_size, statistics: statistics.0 });
            metrics.cycle_count.push(MetricResult { srs_size, circuit_size, statistics: statistics.1 });
        }
    }

    // Print the results in JSON format
    println!("{}", to_string(&metrics).unwrap());
}

/// Generates the SRS (Structured Reference String) for the given size.
///
/// # Arguments
/// * `dir_path` - The directory path where the SRS will be generated.
/// * `srs_size` - The size of the SRS.
/// * `srs_out_path` - The output path where the SRS will be stored.
fn generate_srs(dir_path: &Path, srs_size: i32, srs_out_path: &PathBuf) {
    let srs_gen_output = Command::new("srs_gen")
        .current_dir(dir_path)
        .args(["--size", &srs_size.to_string(), "--output", srs_out_path.to_str().unwrap()])
        .output()
        .expect("Failed to generate SRS");

    if !srs_gen_output.status.success() {
        eprintln!("Gen SRS failed with error: {}\n", String::from_utf8_lossy(&srs_gen_output.stderr));
    } else {
        println!("Gen SRS succeeded with output: {}\n", String::from_utf8_lossy(&srs_gen_output.stderr));
    }
}

/// Tests the verifier with the given parameters and returns the cycle count and binary size.
///
/// # Arguments
/// * `dir_path` - The directory path where the test will be conducted.
/// * `crate_name` - The name of the crate being tested.
/// * `srs_out_path` - The output path where the SRS is stored.
/// * `verifier_contract_path` - The path where the verifier contract will be generated.
/// * `verifier_contract_built_path` - The path where the built verifier contract will be stored.
/// * `circuit_size` - The size of the circuit being tested.
///
/// # Returns
/// A tuple containing the cycle count and the binary size.
fn test_contract_with_params(
    dir_path: &Path,
    crate_name: &str,
    srs_out_path: &PathBuf,
    verifier_contract_path: &PathBuf,
    verifier_contract_built_path: &PathBuf,
    circuit_size: i32,
) -> (Cycle, usize) {
    let equation = format!("x^{}=1", circuit_size);
    let witness = "x=1";

    //Generate contract binary
    generate_verifier_contract(dir_path, crate_name, srs_out_path, &equation, verifier_contract_path);
    build_verifier_contract(verifier_contract_path);

    // Generate and serialize the proof
    let parser = init_parser_with_witnesses(witness);
    let compiled_circuit = parser.parse(&equation).compile().unwrap();
    let srs = Srs::deserialize_uncompressed_unchecked(&fs::read(srs_out_path).unwrap()[..]).unwrap();
    let proof = prover::generate_proof::<Sha256>(&compiled_circuit, srs);

    let mut proof_bytes = Vec::new();
    proof.serialize_uncompressed(&mut proof_bytes).unwrap();

    contract_test(proof_bytes.into(), verifier_contract_built_path.to_str().unwrap())
}

/// Generates the verifier contract.
///
/// # Arguments
/// * `dir_path` - The directory path where the verifier contract will be generated.
/// * `crate_name` - The name of the crate being tested.
/// * `srs_out_path` - The output path where the SRS is stored.
/// * `equation` - The equation to be used in the verifier.
/// * `verifier_contract_path` - The path where the verifier contract will be generated.
fn generate_verifier_contract(
    dir_path: &Path,
    crate_name: &str,
    srs_out_path: &PathBuf,
    equation: &str,
    verifier_contract_path: &PathBuf,
) {
    let verifier_gen_output = Command::new("verifier_gen")
        .current_dir(dir_path)
        .args([
            "--crate-name", crate_name,
            "--srs", srs_out_path.to_str().unwrap(),
            "--equation", equation,
            "--output", verifier_contract_path.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to generate verifier contract");

    if !verifier_gen_output.status.success() {
        eprintln!("Gen verifier contract failed with error: {}\n", String::from_utf8_lossy(&verifier_gen_output.stderr));
    } else {
        println!("Gen verifier contract succeeded with output: {}\n", String::from_utf8_lossy(&verifier_gen_output.stderr));
    }
}

/// Builds the verifier contract.
///
/// # Arguments
/// * `verifier_contract_path` - The path where the verifier contract is generated.
fn build_verifier_contract(verifier_contract_path: &PathBuf) {
    let verifier_build_output = Command::new("make")
        .current_dir(verifier_contract_path)
        .arg("build")
        .output()
        .expect("Failed to build the verifier contract");

    if !verifier_build_output.status.success() {
        eprintln!("Build failed with error: {}\n", String::from_utf8_lossy(&verifier_build_output.stderr));
    } else {
        println!("Build succeeded with output: {}\n", String::from_utf8_lossy(&verifier_build_output.stderr));
    }
}

/// Sets up the environment by building the project.
fn setup() {
    let install_output = Command::new("make")
        .arg("install")
        .output()
        .expect("Failed to build the project");

    if !install_output.status.success() {
        eprintln!("Install failed with error: {}\n", String::from_utf8_lossy(&install_output.stderr));
    } else {
        println!("Install succeeded with output: {}\n", String::from_utf8_lossy(&install_output.stderr));
    }
}

/// Initializes the parser with given witnesses.
///
/// # Arguments
/// * `input` - A string containing the witnesses.
///
/// # Returns
/// A `Parser` instance with the witnesses added.
fn init_parser_with_witnesses(input: &str) -> Parser {
    let mut parser = Parser::default();
    input.split(';').for_each(|key_value| {
        let key_value: Vec<&str> = key_value.split('=').map(|s| s.trim()).collect();
        assert_eq!(key_value.len(), 2);
        parser.add_witness(key_value[0], Fr::from(key_value[1].parse::<i32>().unwrap()));
    });
    parser
}

/// Calculates statistics for size and cycle results.
///
/// # Arguments
/// * `size_results` - A vector containing the size results.
/// * `cycle_results` - A vector containing the cycle results.
/// * `iterations` - The number of iterations the test was run.
/// * `f95_index` - The index corresponding to the 95th percentile.
/// * `f99_index` - The index corresponding to the 99th percentile.
///
/// # Returns
/// A tuple containing `Statistics` for size and cycle results.
fn calculate_statistics(
    size_results: &Vec<usize>,
    cycle_results: &Vec<Cycle>,
    iterations: u64,
    f95_index: usize,
    f99_index: usize,
) -> (Statistics, Statistics) {
    let min_size = *size_results.first().unwrap();
    let max_size = *size_results.last().unwrap();
    let average_size = size_results.iter().sum::<usize>() as f64 / iterations as f64;
    let f95_size = size_results[f95_index];
    let f99_size = size_results[f99_index];

    let size_stats = Statistics {
        min: min_size as f64,
        max: max_size as f64,
        average: average_size,
        f95: f95_size as f64,
        f99: f99_size as f64,
    };

    let min_cycle = *cycle_results.first().unwrap();
    let max_cycle = *cycle_results.last().unwrap();
    let average_cycle = cycle_results.iter().sum::<Cycle>() as f64 / iterations as f64;
    let f95_cycle = cycle_results[f95_index];
    let f99_cycle = cycle_results[f99_index];

    let cycle_stats = Statistics {
        min: min_cycle as f64,
        max: max_cycle as f64,
        average: average_cycle,
        f95: f95_cycle as f64,
        f99: f99_cycle as f64,
    };

    (size_stats, cycle_stats)
}
