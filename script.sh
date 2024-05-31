# create cpi: done
cd verifier_client
cargo run
cd ..


CRATE="$1"

# Define the source directory
SOURCE_DIR="local_script/src"

# Define the destination directory
DESTINATION_DIR="contracts/$CRATE/src"


export CRATE
export TEMPLATE_TYPE="--git"
export TEMPLATE_REPO="https://github.com/cryptape/ckb-script-templates"
export TEMPLATE="contract"
export DESTINATION="contracts"

# Run the generate command
make generate
sleep 2

# Copy all files from local_script into contracts/CRATE/src
cp -r "$SOURCE_DIR"/* "$DESTINATION_DIR"

DEPENDENCY_FILE="local_script/dependency.txt"
CARGO_FILE="contracts/$CRATE/Cargo.toml"

# Check if the dependency file exists
if [ ! -f "$DEPENDENCY_FILE" ]; then
    echo "Error: dependency file '$DEPENDENCY_FILE' not found."
    exit 1
fi

# Check if the Cargo.toml file exists
if [ ! -f "$CARGO_FILE" ]; then
    echo "Error: Cargo.toml file '$CARGO_FILE' not found."
    exit 1
fi

DEPENDENCIES=$(<"$DEPENDENCY_FILE")
sed -i "/\[dependencies\]/ r $DEPENDENCY_FILE" "$CARGO_FILE"

rm -rf build

make build

offckb deploy --target ./build/release/

offckb deployed-scripts