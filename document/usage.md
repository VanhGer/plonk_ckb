# Project Name

## Quick start

### Prerequisites

Before you begin, ensure you have met the following requirements:

- **Docker**: Make sure Docker is installed on your machine. You can download it from [here](https://docs.docker.com/get-docker/).

### How to Run

Follow these steps to run the project:

1. **Run the docker compose file**:
   Open your terminal and navigate to the directory containing the Docker compose file. Run the following command to get Docker started:
   ```sh
   docker compose up -d --build
2. **Run the Verifier**:
   Run the following command to build, deploy and test the verifier on devnet environment:
    ```sh
    docker compose exec -it plonk-verifier ./script.sh run <size-of-srs> <contract-name> <equation> <witnesses> <rpc_url>
    ```
   Where:\
   size-of-srs:     The size of the circuit\
   contract-name:   The name of the crate\
   equation:        The equation to use\
   witnesses:       The witnesses to use\
   rpc_url:         The url of the rpc

### Example:
Here is an example of how to run the verifier script inside the Docker container:
1. **Run the Verifier with parameters**:
   ```sh
   docker compose exec -it plonk-verifier ./script.sh run 1024 plonk_verifier "x^3 + x + 5 = 35" "x=3" "http://127.0.0.1:8114"
   ```
   This command will run the script.sh with the provided parameters, which will configure the verifier to check the equation x^3 + x + 5 = 35 with x = 3 and connect to the specified URL.

### Notes:

-  Ensure you have an active internet connection while building and running the Docker container as it requires downloading dependencies and scripts.
- You can modify the parameters passed to the `script.sh` to suit your specific use case.
- For more details on how to use offckb, refer to its [official documentation](https://github.com/retricsu/offckb).

### Troubleshooting:
If you encounter any issues:

- Docker Issues: Refer to the official Docker documentation for troubleshooting Docker-related problems.
- Script Issues: Ensure that the URL and parameters provided to the script are correct and accessible.

## Custom usage

To use this project, follow these steps:


1. **Run the `offckb` Node:**
   ```bash
   offckb node
   ```

2. **Make the Script Executable:**

   ```bash
   chmod +x script.sh
   ```

3. **Run the Trusted Setup**:

   ```bash
	./setup.sh <srs_size>
   ```


4. **Run the Verifier Script to Create and Deploy the Smart Contract:**

   ```bash
   ./verifier_script.sh <contract-name> <equation>
   ```

   After running this command, the smart contract used to verify the proof of the equation will be deployed. For example:

   ```bash
   ./verifier_script.sh "abc" "x^3+x+5=35"
   ```    


5. **Run the Prover Script to Send the Proof to the Smart Contract and Wait for Verification:**

   ```bash
   ./prover_script.sh <contract-name> <equation> <witness>
   ```    

   For example:

   ``` bash
   ./prover_script.sh "abc" "x^3+x+5=35" "x=3"
   ```



These steps provide a clear guide for running the node, preparing the scripts, deploying the verifier contract, and sending proofs to the smart contract for verification. Adjust the placeholders (`<size-of-srs>`, `<contract-name>`, `<equation>`, `<witness>`) with your actual values.
