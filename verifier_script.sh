set -e
CRATE="$1"
EQUATION="$2"


# install necessary tools
printf "Installing tools\n"
make install

printf "Generating verifier_contracts\n"
rm -Rf debug/"$CRATE"
mkdir debug/"$CRATE"
#srs_gen --size "$CIRCUIT_SIZE" --output debug/"$CRATE"/srs.bin
cp srs.bin debug/"$CRATE"/
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

printf "Contract "$CRATE" is deployed successfully!\n"