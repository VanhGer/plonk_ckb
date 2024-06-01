use std::fs;
use clap::Parser;
use plonk::common_preprocessed_input::cpi_parser::CPIParser;

fn main() -> Result<(), std::io::Error> {
    // Parse command-line arguments
    let args = Args::parse();

    // Generate the Common Preprocessed Input (CPI) and write it to a file
    eprintln!("Generating CPI for the equation: {:#?}", args.equation);

    let cpi_bytes = CPIParser::default().compute_common_preprocessed_input(args.equation.as_str());

    let output_content = format!(
        "pub const COMMON_PREPROCESSED_INPUT: [u8; {}] = {:?};",
        cpi_bytes.len(),
        &cpi_bytes
    );

    // Write the generated CPI to the specified output file
    fs::write(&args.output, output_content)?;

    eprintln!("CPI generated successfully! Output path: {:#?}", args.output);
    Ok(())
}

/// Generate Common Preprocessed Input (CPI)
///
/// # Example:
/// ```sh
/// ./cpi --equation "<equation>" --output "<file-path>"
/// ```
///
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The equation to process
    #[clap(long, value_name = "equation", default_value = "x + y*y + 3*z = 10")]
    equation: String,

    /// The output file path
    #[clap(
        long,
        value_name = "output",
        default_value = "../local_script/src/common_processed_input_const.rs"
    )]
    output: String,
}
