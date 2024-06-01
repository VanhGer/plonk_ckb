use ark_serialize::CanonicalSerialize;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use clap::Parser;
use include_dir::{include_dir, Dir, DirEntry};
use toml::Value;

use plonk::common_preprocessed_input::cpi_parser::CPIParser;

static ASSETS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../contract_templates/plonk_verifier");

/// Command-line argument parser for the verifier contract generator
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// The name of the crate to be created
    #[clap(long, value_name = "crate_name", default_value = "my_verifier")]
    crate_name: String,

    /// The equation to process
    #[clap(long, value_name = "equation", default_value = "x + y*y + 3*z = 10")]
    equation: String,

    /// The output folder path
    #[clap(long, value_name = "output", default_value = "my_verifier")]
    output: String,

    /// The SRS (Structured Reference String) binary file path
    #[clap(long, value_name = "srs", default_value = "srs.bin")]
    srs: String,
}

/// Main function for generating verifier contracts
///
/// Parses command-line arguments, generates the Common Preprocessed Input (CPI),
/// writes it to a file, creates the output directory, extracts embedded assets to the output path,
/// replaces the package name in the Cargo.toml file, and copies the SRS binary to the output path.
fn main() -> Result<(), std::io::Error> {
    // Parse command-line arguments
    let args = Args::parse();

    // Generate the Common Preprocessed Input (CPI) and write it to a file
    println!(
        "Generating verifier contracts for the equation: {:#?}",
        args.equation
    );

    let cpi = CPIParser::default()
        .compute_common_preprocessed_input(&args.equation)
        .expect("Failed to compute CPI");
    let mut cpi_bytes = Vec::new();
    cpi.serialize_uncompressed(&mut cpi_bytes).expect("Failed to serialize CPI");

    // Specify the output path where the folder should be generated
    let output_path = Path::new(&args.output);

    // Create the output directory
    fs::create_dir_all(output_path)?;

    // Recursively extract the embedded directory to the output path
    extract_dir(&ASSETS_DIR, output_path)?;

    // Replace the package name in Cargo.toml
    replace_package_name(output_path, &args.crate_name);

    // Write the CPI bytes to a file
    let cpi_file_path = output_path.join("src/cpi.bin");
    let mut file = File::create(&cpi_file_path)?;
    file.write_all(&cpi_bytes)?;

    // Copy the SRS binary to the output path
    let src = Path::new(&args.srs);
    let srs_dest = output_path.join("src/srs.bin");
    fs::copy(src, &srs_dest)?;

    println!("Verifier contract generated successfully! Output path: {:?}", args.output);
    Ok(())
}

/// Recursively extract embedded directory contents to the specified output path
///
/// Iterates through the entries of the given directory and writes files to the specified output path.
/// If an entry is a directory, it creates the directory and recursively calls itself to extract its contents.
///
/// # Arguments
///
/// * `dir` - The embedded directory to extract
/// * `output_path` - The path where the directory contents will be written
///
/// # Returns
///
/// A Result indicating success or an error
fn extract_dir(dir: &Dir, output_path: &Path) -> std::io::Result<()> {
    for entry in dir.entries() {
        let path = output_path.join(entry.path());
        match entry {
            DirEntry::Dir(d) => {
                fs::create_dir_all(&path)?;
                extract_dir(d, output_path)?;
            }
            DirEntry::File(f) => {
                fs::write(&path, f.contents())?;
            }
        }
    }
    Ok(())
}

/// Replace the package name in the Cargo.toml file with the specified crate name
///
/// Reads the Cargo.toml file, updates the package name with the provided crate name,
/// and writes the updated content back to the Cargo.toml file.
///
/// # Arguments
///
/// * `output_path` - The path to the output directory containing the Cargo.toml file
/// * `crate_name` - The new crate name to set in the Cargo.toml file
///
/// # Panics
///
/// Panics if it fails to read, parse, or write the Cargo.toml file
fn replace_package_name(output_path: &Path, crate_name: &str) {
    let cargo_toml_path = output_path.join("Cargo.toml");

    // Read the Cargo.toml file
    let cargo_toml_content =
        fs::read_to_string(&cargo_toml_path).expect("Failed to read Cargo.toml file");

    // Parse the Cargo.toml content
    let mut cargo_toml_value: Value = cargo_toml_content
        .parse()
        .expect("Failed to parse Cargo.toml content");

    // Update the package name
    if let Some(package) = cargo_toml_value.get_mut("package") {
        if let Some(name) = package.get_mut("name") {
            *name = Value::String(crate_name.to_string());
        }
    }

    // Convert the updated TOML value back to a string
    let new_cargo_toml_content = toml::to_string(&cargo_toml_value)
        .expect("Failed to convert updated Cargo.toml content to string");

    // Write the updated content back to the Cargo.toml file
    fs::write(cargo_toml_path, new_cargo_toml_content)
        .expect("Failed to write updated content to Cargo.toml file");

    println!("Package name updated successfully.");
}
