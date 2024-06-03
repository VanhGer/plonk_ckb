# Usage

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
