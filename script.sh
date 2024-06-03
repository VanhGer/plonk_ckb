set -e
CIRCUIT_SIZE="$1"
CRATE="$2"
EQUATION="$3"
WITNESSES="$4"


# install necessary tools
printf "Installing tools\n"
make install

printf "Generating verifier_contracts\n"
rm -Rf debug/"$CRATE"
mkdir debug/"$CRATE"
srs_gen --size "$CIRCUIT_SIZE" --output debug/"$CRATE"/srs.bin
verifier_gen --crate-name "$CRATE" --srs debug/"$CRATE"/srs.bin --output debug/"$CRATE"/verifier_contracts --equation "$EQUATION"
printf "\n"

printf "Building verifier_contracts\n"
cd debug/"$CRATE"/verifier_contracts && chmod 777 ./scripts/find_clang && make build
printf "\n"

printf "contracts size\n"
ls -lh ./target/riscv64imac-unknown-none-elf/release/"$CRATE"
printf "\n"

printf "Deploying verifier_contracts\n"
offckb deploy --target ./target/riscv64imac-unknown-none-elf/release
printf "\n"

cd ../../..

printf "Generating prover_config\n"
crate_upper=$(echo "$CRATE" | tr '[:lower:]' '[:upper:]')
json_input=$(offckb deployed-scripts | sed -n '/{/,$p')
code_hash=$(echo "$json_input" | jq -r ".$crate_upper.CODE_HASH")
tx_hash=$(echo "$json_input" | jq -r ".$crate_upper.TX_HASH")
# Write to a TOML file
output_file="${CRATE}_output.toml"
cat <<EOL > $output_file
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
printf "\n"

printf "Running prover\n"
prover --config-path $output_file
printf "\n"
