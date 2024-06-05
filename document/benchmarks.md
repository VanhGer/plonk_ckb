# Benchmarks

In the [Nervos CKB](https://www.nervos.org/) network, users must pay for data storage, transaction fees, and computational resources. The cost for data storage is directly proportional to the size of the transaction, requiring a certain number of CKB tokens. Computational resources, measured in **cycles**, incur additional CKB charges based on the amount needed to verify a transaction.

On the mainnet, the value of `MAX_BLOCK_BYTES` is `579 000`, and `MAX_BLOCK_CYCLES` is `3 500 000 000`.

The deployer should pay for storing his contract on-chain. So in our project, in order to reduce the binary size, we modify the *Cargo.toml* :

```toml
[profile.release]  
overflow-checks = true  
strip = true  
# codegen-units: greater than 0, default 16
codegen-units = 1
# lto: true, "thin", false(default)
lto=true  
Zlocation-detail="none"
# opt-level: 0, 1, 2, 3(default), "s", "z"
opt-level="z"
```

## Verifier contract benchmarks

We compare our work with [CKB-ZKP](https://github.com/sec-bit/ckb-zkp) work:

Test setup:
- Release mode
- stripped
- Profile: `LTO = true`, `codegen-units = 1`, `panic = "abort"`, `overflow-checks = true`, `opt-level = "z"`;
- Trusted setup: SRS size = 4

| Scheme              | Binary size(Byte) | Execution cost (cycles) |
| ------------------- | ----------------- | ----------------------- |
| Groth16             | 58,288            | 195,535,979             |
| Bulletproofs        | 91,056            | 796,836,045             |
| Marlin              | 132,016           | 500,725,146             |
| Spartan (snark)     | 119,728           | 1,911,833,747           |
| CLINKv2 (kzg10)     | 82,864            | 213,212,113             |
| **Our Work: PLONK** | **97,920**            | **476,103,620**             |

In this project, we use the curve `bls12_381` while **CKB-ZKP** uses `bn_256`, which has an execution cost less than half of that of `bls12_381`, according to [CKB-ZKP](https://github.com/sec-bit/ckb-zkp/blob/master/README.md#curve-benchmark).

