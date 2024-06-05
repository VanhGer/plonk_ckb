#!/bin/bash

set -e

# Function to show help
show_help() {
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  run <circuit_size> <crate> <equation> <witnesses>  Run the full process with the specified parameters"
    echo "  info                                                 Show detailed information about the script"
    echo "  help                                                 Show this help message"
    echo ""
    echo "Options:"
    echo "  circuit_size    The size of the circuit"
    echo "  crate           The name of the crate"
    echo "  equation        The equation to use"
    echo "  witnesses       The witnesses to use"
}

# Function to show info
show_info() {
    echo "This script automates the process of generating, building, and deploying verifier contracts, as well as running a prover."
    echo "Ensure all necessary tools and dependencies are installed before running the script."
}

# Main process
run_process() {
    CIRCUIT_SIZE="$1"
    CRATE="$2"
    EQUATION="$3"
    WITNESSES="$4"
    RPC="$5"

    if [ -z "$CIRCUIT_SIZE" ] || [ -z "$CRATE" ] || [ -z "$EQUATION" ] || [ -z "$WITNESSES" ]; then
        echo "Error: Missing arguments for 'run' command"
        show_help
        exit 1
    fi

    # install necessary tools
#    echo "Installing tools"
#    make install

    echo "Generating verifier_contracts"
    rm -Rf debug/"$CRATE"
    mkdir -p debug/"$CRATE"
    srs_gen --size "$CIRCUIT_SIZE" --output debug/"$CRATE"/srs.bin
    verifier_gen --crate-name "$CRATE" --srs debug/"$CRATE"/srs.bin --output debug/"$CRATE"/verifier_contracts --equation "$EQUATION"
    echo ""

    echo "Building verifier_contracts"
    cd debug/"$CRATE"/verifier_contracts && chmod 777 ./scripts/find_clang && make build
    echo ""

    echo "contracts size"
    ls -lh ./target/riscv64imac-unknown-none-elf/release/"$CRATE"
    echo ""

    echo "Deploying verifier_contracts"
    offckb deploy --target ./target/riscv64imac-unknown-none-elf/release
    echo ""

    cd ../../..

    echo "Generating prover_config"
    crate_upper=$(echo "$CRATE" | tr '[:lower:]' '[:upper:]')
    json_input=$(offckb deployed-scripts | sed -n '/{/,$p')
    code_hash=$(echo "$json_input" | jq -r ".$crate_upper.CODE_HASH")
    tx_hash=$(echo "$json_input" | jq -r ".$crate_upper.TX_HASH")

    # Write to a TOML file
    output_file="${CRATE}_output.toml"
    cat <<EOL > $output_file
sender_key = "ace08599f3174f4376ae51fdc30950d4f2d731440382bb0aa1b6b0bd3a9728cd"
receiver = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqvm52pxjfczywarv63fmjtyqxgs2syfffq2348ad"
ckb_rpc = "$RPC"
equation = "$EQUATION"
verifier_code_hash = "$code_hash"
tx_hash = "$tx_hash"
witnesses = "$WITNESSES"
srs_path = "debug/$CRATE/srs.bin"
EOL
    echo "TOML file created: $output_file"
    echo ""

    echo "Running prover"
    prover --config-path $output_file
    echo ""
}

# Parse command line arguments
case "$1" in
    run)
        shift
        run_process "$@"
        ;;
    info)
        show_info
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "Error: Unknown command: $1"
        show_help
        exit 1
        ;;
esac
