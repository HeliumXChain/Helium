# HELIUM: A DECENTRALIZED PROTOCOL FOR VERIFIABLE MACHINE LEARNING ON PROOF OF USEFUL WORK (PoUW)

**Date**: 2026-03-22
**Version**: 0.1.0-kuro
**Status**: Research Phase L1 (Draft)

## 1. ABSTRACT
Helium is a Layer 1 blockchain designed to decentralize the global AI compute market. By replacing arbitrary cryptographic hashing (Proof of Wasted Work) with supervised machine learning training (Proof of Useful Work), Helium aligns network security with the production of high-value AI assets. Through Zero-Knowledge Machine Learning (ZK-ML) and hardware-level attestation (TPM 2.0), Helium ensures that all compute performed is verifiable, confidential, and sovereign.

## 2. INTRODUCTION: THE COMPUTE MONOPOLY
The current AI landscape is bottlenecked by centralized cloud providers (AWS, GCP, Azure). This leads to:
1.  **High Concentration Risk**: Global AI progress depends on a handful of server clusters.
2.  **Cost Inefficiency**: Significant markups on GPU compute hours.
3.  **Privacy Concerns**: Trusting centralized entities with proprietary model weights and datasets.

Helium proposes a trustless alternatively: an open marketplace for GPU compute where every hash performed contributes to training the next generation of AI models.

## 3. PROOF OF USEFUL WORK (PoUW)
In Helium, the consensus mechanism is inherently tied to AI training tasks.
*   **The Mining Cycle**: A block is mined when a node successfully completes one training epoch of a client-submitted job and generates a valid proof.
*   **Difficulty Adjustment**: Network difficulty is adjusted based on the computational complexity of the pending job queue and the target block time.
*   **Economic Utility**: Unlike traditional PoW where energy is "burned" for security, Helium energy is "invested" in model weights.

## 4. VERIFIABLE COMPUTE (ZK-ML)
Trusting a decentralized node to train a model requires verification without disclosing data.
*   **Zero-Knowledge Proofs**: Helium utilizes ZK-SNARKs to prove that a model's loss function decreased by the expected amount during an epoch, without revealing the training data or the resulting weights.
*   **Hardware Attestation**: Using TPM 2.0 (Trusted Platform Module), nodes provide a signed cryptographic identity of the hardware (e.g., NVIDIA H100) and a Merkle Tree proof of the execution trace.
*   **Verification Pipeline**: A set of dedicated "Validator Nodes" verifies the ZK-proofs and hardware signatures before a block is committed to the chain.

## 5. TOKENOMICS: THE $HLM ECONOMY
The Helium ecosystem is powered by the $HLM token, which serves three roles:
1.  **Settlement**: Clients pay for compute jobs in $HLM.
2.  **Reward**: Node operators earn $HLM for mining blocks (verified training epochs).
3.  **Staking**: Validators and miners must stake $HLM to participate, with slashing penalties for dishonest attestation.

## 6. ROADMAP & GOVERNANCE
*   **Q1 2026 (Current)**: Research L1 - Mathematical validation and L1 architecture specs.
*   **Q2 2026**: Alpha Simulation - Local node trials and PoUW simulation.
*   **Q3 2026**: Testnet - Global distributed compute sandbox.
*   **Q4 2026**: Mainnet Genesis - Launch of the sovereign compute layer.

## 7. CONCLUSION
Helium is not just a blockchain; it is the infrastructure for a sovereign AI future. By merging the security of Bitcoin with the utility of OpenAI, Helium creates a sustainable, decentralized engine for global intelligence.
