use std::fs;
use plonk::common_preprocessed_input::cpi_parser::CPIParser;
use crate::initialize::{EQUATION};

mod initialize;
fn main() {
    let equation = EQUATION;

    // compute cpi and write it into a file
    let cpi_bytes = CPIParser::default().compute_common_preprocessed_input(equation);
    let str = format!(
        "pub const COMMON_PROCESSED_INPUT:[u8;{}] = {:?};",
        cpi_bytes.len(),
        &cpi_bytes
    );
    fs::write("../local_script/src/common_processed_input_const.rs", str).expect("write failed");
}
