# Plonk CKB

## Quick start

### Prerequisites

Before you begin, ensure you have met the following requirements:

- **Docker**: Make sure Docker is installed on your machine. You can download it from [here](https://docs.docker.com/get-docker/).

### How to Run

Follow these steps to run the project:

1. **Setup running environment**
   - **Using docker**
      1. **Build the docker file**:
         Open your terminal and navigate to the directory containing the Docker file. Run the following command to build the image:
         ```sh
         docker build -t zkp_ckb_tool .
         ```
     2. **Spawn the container**:
         Run the following command to spawn and then access its terminal:
          ```sh
          docker run -d --name=zkp_ckb_tool_container zkp_ckb_tool
          docker exec -it zkp_ckb_tool_container /bin/bash
          ```
   - **Using local**
     1. **Install offckb**:
        ```sh
        npm install -g @offckb/cli
        ```
     2. **Run offckb**:
        ```sh
        offckb node
        ```
     3. **Install CLI tools**:
        ```sh
        cargo install --git https://github.com/VanhGer/plonk_ckb/ cli --branch main
        ```
     4. **Download and set permissions for the script**:
        ```sh
        curl -o ./script.sh https://raw.githubusercontent.com/VanhGer/plonk_ckb/main/script.sh
        chmod 755 ./script.sh
        ```
2. **The full flow contains 3 phases**:
   - Generate trusted setup: this will generate common parameters that both prover and verifier need to know about.
   ```
    ./script.sh gen-srs <size-of-srs> 
    
   ```
   - Generate the verifier contract and then deploy it.
   ```
   ./script.sh gen-verifier <contract-name> <equation> [optional] <rpc-ckb>

   ```
   - Generate ZK proof and submit the proof to the verifier contract to verify it.
   ```
   ./script.sh prover <contract-name> <equation> <witnesses> [optional] <rpc-ckb>

   ```
   Where:\
   `size-of-srs`:     The size of the SRS (Structured Reference String)\
   `contract-name`:   The name of the crate\
   `equation`:        The equation to use\
   `witnesses`:       The witnesses to use\
   `rpc`:             (optional) The RPC URL of the CKB node\

### Example:
Here is an example of how to run the verifier script inside the Docker container:
1. **Run the Verifier with parameters**:
   ```sh
   ./script.sh gen-srs 1024
   ./script.sh gen-verifier plonk_verifier "x^3 + x + 5 = 35" "x=3"
   ./script.sh prover plonk_verifier "x^3 + x + 5 = 35" "x=3"
   ```
   This command will run the script.sh with the provided parameters, which will configure the verifier to check the equation x^3 + x + 5 = 35 with x = 3 and connect to the specified URL.

### Notes:

- Ensure you have an active internet connection while building and running the Docker container as it requires downloading dependencies and scripts.
- You can modify the parameters passed to the `script.sh` to suit your specific use case.
- For more details on how to use offckb, refer to its [official documentation](https://github.com/retricsu/offckb).

### Troubleshooting:
If you encounter any issues:

- Docker Issues: Refer to the official Docker documentation for troubleshooting Docker-related problems.
- Script Issues: Ensure that the URL and parameters provided to the script are correct and accessible.
