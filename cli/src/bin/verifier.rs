use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use ark_serialize::CanonicalSerialize;
use clap::Parser;
use include_dir::{Dir, DirEntry, include_dir};
use toml::Value;
use plonk::common_preprocessed_input::cpi_parser::CPIParser;

static ASSETS_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/../contract_templates/plonk_verifier");

/// Generate Verifier Contract
///
/// # Example:
/// ```sh
/// ./verifier_gen --equation "<equation>" --output "<file-path>"
/// ```
///
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// The resulted crate name
    #[clap(
        long,
        value_name = "crate_name",
        default_value = "my_verifier"
    )]
    crate_name: String,

    /// The equation to process
    #[clap(long, value_name = "equation", default_value = "x + y*y + 3*z = 10")]
    equation: String,

    /// The output folder path
    #[clap(
        long,
        value_name = "output",
        default_value = "my_verifier"
    )]
    output: String,

    /// The srs bin path
    #[clap(
        long,
        value_name = "srs",
        default_value = "srs.bin"
    )]
    srs: String,
}

fn main() -> Result<(), std::io::Error> {
    // Parse command-line arguments
    let args = Args::parse();

    // Generate the Common Preprocessed Input (CPI) and write it to a file
    println!("Generating verifier contracts for the equation: {:#?}", args.equation);

    let cpi = CPIParser::default().compute_common_preprocessed_input(args.equation.as_str()).unwrap();
    let mut cpi_bytes = Vec::new();
    cpi.serialize_uncompressed(&mut cpi_bytes).unwrap();

    // Specify the output path where the folder should be generated
    let output_path = Path::new(&args.output);

    // Create the output directory
    fs::create_dir_all(output_path)?;

    // Recursively extract the embedded directory to the output path
    extract_dir(&ASSETS_DIR, output_path)?;

    replace_package_name(output_path, &args.crate_name);

    let mut file = File::create(format!("{}/src/cpi.bin", &args.output))?;
    file.write_all(&cpi_bytes)?;

    let src = Path::new(&args.srs);
    let srs_path = format!("{}/src/srs.bin", &args.output);
    let dest = Path::new(&srs_path);

    // Copy the file from src to dest
    fs::copy(&src, &dest)?;

    println!("Verifier contract generated successfully! Output path: {:#?}", args.output);
    Ok(())
}


// Function to extract embedded directory
fn extract_dir(dir: &Dir, output_path: &Path) -> std::io::Result<()> {
    for entry in dir.entries() {
        let path = output_path.join(entry.path());
        match entry {
            DirEntry::Dir(d) => {
                fs::create_dir_all(&path)?;
                extract_dir(d, &output_path)?;
            }
            DirEntry::File(f) => {
                fs::write(path, f.contents())?;
            }
        }
    }
    Ok(())
}

fn replace_package_name(output_path: &Path, crate_name: &str) {
    let cargo_toml_path = output_path.join("Cargo.toml");

    // Read the Cargo.toml file
    let cargo_toml_content = fs::read_to_string(&cargo_toml_path)
        .expect("Failed to read Cargo.toml file");

    // Parse the Cargo.toml content
    let mut cargo_toml_value: Value = cargo_toml_content.parse()
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
    println!("new_cargo_toml_content {:#?}", new_cargo_toml_content);
    // Write the updated content back to the Cargo.toml file
    fs::write(cargo_toml_path, new_cargo_toml_content)
        .expect("Failed to write updated content to Cargo.toml file");

    println!("Package name updated successfully.");
}
