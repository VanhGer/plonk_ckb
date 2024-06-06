# PLONK CKB

## Overview

This project is a zero-knowledge proof system, where the prover generates proof off-chain, and the verifier verifies it
on-chain by invoking the verifier smart contract.

## Features

- **Prover**: Generates cryptographic proofs off-chain.
- **Verifier**: Create a verifier smart contract to verify the proof.
- **Protocol**: This proof system utilizes the [PLONK](https://eprint.iacr.org/2019/953.pdf) protocol for efficient and
  scalable zero-knowledge proofs.

## Getting Started

To get started with this project, clone the repository and install the required dependencies.

```bash
# clone the repo
git clone https://github.com/VanhGer/plonk_ckb.git

# install offckb
npm install -g @offckb/cli
```

Then, make sure that you have all tools:

- [make](https://www.tutorialspoint.com/unix_commands/make.htm),[sed](https://www.gnu.org/software/sed/),[bash](https://www.gnu.org/software/bash/),[sha256sum](https://linux.die.net/man/1/sha256sum)
  and others Unix utilities.
- [Clang 16+](https://releases.llvm.org/16.0.0/tools/clang/docs/ReleaseNotes.html)
- [Rust](https://www.rust-lang.org/)
- [Cargo-generate](https://github.com/cargo-generate/cargo-generate)

## Usage

You should read the usage of this project [here](documents/usage.md).

## Roadmap

- [x] Implement PLONK protocol with KZG commitment
- [x] Generate proof off-chain and verify it on-chain.
- [ ] Connect to [StarkNet](https://www.starknet.io/).

## Benchmarks

You can see our benchmarks [here](documents/benchmarks.md).

## Contributing

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also
simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

## Contact

VanhGer - [@vanhg](https://twitter.com/vanhger) - vietanhpg2003@gmail.com

## Acknowledgments

- [CKB-ZKP](https://github.com/sec-bit/ckb-zkp): Zero-knowledge proofs toolkit for CKB.
- [Sota-ZK-implementation](https://github.com/sota-zk-labs/zkp-implementation): Implementation of ZKP protocols. 


