# Use the official Ubuntu 22.04 as the base image
FROM ubuntu:22.04

# Update package lists and install required packages
RUN apt update && apt install -y curl build-essential lsb-release software-properties-common pkg-config libssl-dev wget jq

# Set the working directory to /root
WORKDIR /root

# Install Rust with the nightly toolchain and the RISC-V target
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y --default-toolchain nightly --target riscv64imac-unknown-none-elf

# Update PATH environment variable to include Cargo binaries
ENV PATH="/root/.cargo/bin:$PATH"

# Install CLI tools from the specified Git repository and branch using Cargo
RUN cargo install --git https://github.com/VanhGer/plonk_ckb/ cli --branch main

# Download and set permissions for the script
RUN curl -o ./script.sh https://raw.githubusercontent.com/VanhGer/plonk_ckb/main/script.sh
RUN chmod 755 ./script.sh

# Install LLVM for compiling into RISC-V64 environment
RUN wget https://apt.llvm.org/llvm.sh && chmod +x llvm.sh && ./llvm.sh 16 && rm llvm.sh

# Download and install Node.js
RUN curl -sL https://deb.nodesource.com/setup_20.x | bash -
RUN apt-get install -y nodejs

# Install the offckb CLI tool globally using npm
RUN npm install -g @offckb/cli

# Change the parameter here to configure the verifier
# Example: ./script.sh run 1024 plonk_verifier "x^3 + x + 5 = 35" "x=3" "http://127.0.0.1:8114"

# Set the default command to start the offckb node
CMD ["offckb", "node"]
