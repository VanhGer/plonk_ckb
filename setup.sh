set -e
CIRCUIT_SIZE="$1"

printf "Installing tools\n"
make install

printf "generate trusted setup"
srs_gen --size "$CIRCUIT_SIZE" --output srs.bin