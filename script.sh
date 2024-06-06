#!/bin/bash

set -e

WORKING_DIR=$(pwd)

# Function to show help
show_help() {
    echo "Usage: $0 <command> [arguments]"
    echo ""
    echo "Commands:"
    echo "  gen-srs         Generate SRS via trusted setup"
    echo "  gen-verifier    Generate verifier contract"
    echo "  prover          Run prover"
    echo "  info            Show detailed information about the script"
    echo "  help            Show this help message"
    echo "Arguments:"
    echo "  circuit_size    The size of the circuit"
    echo "  crate           The name of the crate"
    echo "  equation        The equation to use"
    echo "  witnesses       The witnesses to use"
    echo "  ckb_rpc         The RPC URL of the CKB node"
}

# Function to show info
show_info() {
    echo "This script automates the process of generating, building, and deploying verifier contracts, as well as running a prover."
    echo "Ensure all necessary tools and dependencies are installed before running the script."
}

# Main process
run_process() {
    check_cli_exist
    COMMAND="$1"

    shift
    SRS_SIZE="$1"
    CRATE="$2"
    EQUATION="$3"
    WITNESSES="$4"

    case "$COMMAND" in
        gen-srs)
            if [ -z "$SRS_SIZE" ]; then
                echo "Error: Missing arguments for 'run' command"
                show_help
                exit 1
            fi
            run_gen_srs "$@"
            ;;
        gen-verifier)
            if [ -z "$CRATE" ] || [ -z "$EQUATION" ]; then
                echo "Error: Missing arguments for 'gen-verifier' command"
                show_help
                exit 1
            fi
            run_gen_verifier "$@"
            ;;
        prover)
            if [ -z "$CRATE" ] || [ -z "$EQUATION" ] || [ -z "$WITNESSES" ]; then
                echo "Error: Missing arguments for 'prover' command"
                show_help
                exit 1
            fi
            run_prover "$@"
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
}

check_cli_exist() {
    echo "Checking commands exist..."
    # Check if the command exists
    if srs_gen --version >/dev/null 2>&1 && \
       verifier_gen --version >/dev/null 2>&1 && \
       prover --version >/dev/null 2>&1; then
      echo "All commands exist."
    else
      make install
    fi
}

run_gen_srs() {
    echo "Generating structured reference string via trusted setup"
    rm -Rf debug/"$CRATE"
    mkdir -p debug/"$CRATE"
    srs_gen --size "$SRS_SIZE" --output debug/"$CRATE"/srs.bin
    echo ""
}


run_gen_verifier() {
    echo "Building verifier_contracts"
    verifier_gen --crate-name "$CRATE" --srs debug/"$CRATE"/srs.bin --output debug/"$CRATE"/verifier_contracts --equation "$EQUATION"
    export CUSTOM_RUSTFLAGS="--cfg debug_assertions"
    cd debug/"$CRATE"/verifier_contracts && make build
    echo ""

    echo "contracts size"
    ls -lh ./target/riscv64imac-unknown-none-elf/release/"$CRATE"
    echo ""

    echo "Deploying verifier_contracts"
    offckb deploy --target ./target/riscv64imac-unknown-none-elf/release
    echo ""
}

run_prover() {
    echo "Generating prover_config"
    crate_upper=$(echo "$CRATE" | tr '[:lower:]' '[:upper:]')
    json_input=$(offckb deployed-scripts | sed -n '/{/,$p')
    code_hash=$(echo "$json_input" | jq -r ".$crate_upper.CODE_HASH")
    tx_hash=$(echo "$json_input" | jq -r ".$crate_upper.TX_HASH")

    # Write to a TOML file
    output_file="${CRATE}_output.toml"
    cat <<EOL > "$output_file"
sender_key = "ace08599f3174f4376ae51fdc30950d4f2d731440382bb0aa1b6b0bd3a9728cd"
receiver = "ckt1qzda0cr08m85hc8jlnfp3zer7xulejywt49kt2rr0vthywaa50xwsqvm52pxjfczywarv63fmjtyqxgs2syfffq2348ad"
ckb_rpc = "http://127.0.0.1:8114"
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

run_process "$@"
