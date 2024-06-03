set -e

CRATE="$1"
EQUATION="$2"
WITNESSES="$3"

printf "Installing tools\n"
make install

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