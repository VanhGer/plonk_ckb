use std::fs::File;
use std::io::Write;

use ark_serialize::CanonicalSerialize;
use clap::Parser;

use kzg::srs::Srs;

/// This is a tool for generating a Structured Reference String (SRS).
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// The size of the SRS to generate
    #[clap(long, value_name = "size", default_value_t = 100)]
    size: usize,

    /// The output file path where the SRS will be saved
    #[clap(long, value_name = "output", default_value = "srs.bin")]
    output: String,
}

/// Main function for the SRS generator.
///
/// This function parses command-line arguments, generates an SRS of the specified size,
/// serializes it, and writes it to the specified output file.
fn main() -> Result<(), std::io::Error> {
    // Parse command-line arguments
    let args = Args::parse();

    // Generate an SRS of the specified size
    let srs = Srs::new(args.size);

    // Serialize the SRS into a byte vector
    let mut srs_bytes = Vec::new();
    srs.serialize_uncompressed(&mut srs_bytes).expect("Failed to serialize SRS");

    // Write the serialized SRS to the specified output file
    let mut file = File::create(&args.output)?;
    file.write_all(&srs_bytes)?;

    // Notify the user that the SRS was generated successfully
    eprintln!("SRS generated successfully! Output path: {}", args.output);
    Ok(())
}
