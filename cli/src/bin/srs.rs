use std::fs::File;
use std::io::Write;
use clap::Parser;
use kzg::srs::Srs;
use ark_serialize::{CanonicalSerialize};

/// Generate Common Preprocessed Input (CPI)
///
/// # Example:
/// ```sh
/// ./srs-gen --size "<size>" --output "<file-path>"
/// ```
///
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The equation to process
    #[clap(long, value_name = "size", default_value_t = 100)]
    size: usize,

    /// The output folder path
    #[clap(
        long,
        value_name = "output",
        default_value = "srs.bin"
    )]
    output: String,
}

fn main() -> Result<(), std::io::Error> {
    // Parse command-line arguments
    let args = Args::parse();

    let srs = Srs::new(args.size);
    let mut srs_bytes = Vec::new();
    srs.serialize_uncompressed(&mut srs_bytes).unwrap();
    let mut file = File::create(format!("{}", &args.output))?;
    file.write_all(&srs_bytes)?;

    eprintln!("SRS generated successfully! Output path: {:#?}", args.output);
    Ok(())
}