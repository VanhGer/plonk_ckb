# Project Name

## Prerequisites

Before you begin, ensure you have met the following requirements:

- **Docker**: Make sure Docker is installed on your machine. You can download it from [here](https://docs.docker.com/get-docker/).

## How to Run

Follow these steps to run the project:

1. **Run the docker compose file**:
   Open your terminal and navigate to the directory containing the Docker compose file. Run the following command to get Docker started:
   ```sh
   docker compose up -d
2. **Bash into the running Docker container**:
   Run the following command to bash into the Verifier Docker container:
   ```sh
   docker exec -ti plonk-verifier /bin/bash
3. **Run the Verifier**:
   Run the following command to build, deploy and test the verifier on devnet environment:
   ```sh
   ./script.sh run <circuit_size> <crate> <equation> <witnesses>
   ```
   Where:\
   circuit_size:    The size of the circuit"\
   crate:           The name of the crate"\
   equation:        The equation to use"\
   witnesses:       The witnesses to use"

## Example:
Here is an example of how to run the verifier script inside the Docker container:
1. **Run the Verifier with parameters**:
   ```sh
   ./script.sh run 1024 plonk_verifier "x^3 + x + 5 = 35" "x=3" "http://127.0.0.1:8114"
   ```
   This command will run the script.sh with the provided parameters, which will configure the verifier to check the equation x^3 + x + 5 = 35 with x = 3 and connect to the specified URL.

## Notes:

-  Ensure you have an active internet connection while building and running the Docker container as it requires downloading dependencies and scripts.
- You can modify the parameters passed to the `script.sh` to suit your specific use case.
- For more details on how to use offckb, refer to its [official documentation](https://github.com/retricsu/offckb).
   
## Troubleshooting:
If you encounter any issues:

- Docker Issues: Refer to the official Docker documentation for troubleshooting Docker-related problems.
- Script Issues: Ensure that the URL and parameters provided to the script are correct and accessible.



