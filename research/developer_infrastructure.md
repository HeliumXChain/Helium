# HELI-OS: DEVELOPER INFRASTRUCTURE & DOCUMENTATION

**Target Audience**: GPU Miners, AI Researchers, Core Protocol Contributors.

## 1. HELIUM SDK (The Python/Rust Bridge)
For AI researchers to submit jobs to the network.
*   **Repo Name**: `helium-sdk`
*   **Core Functions**:
    *   `helium.submit(model, dataset, target_loss)`: Encapsulates the job submission logic.
    *   `helium.verify(job_id)`: Fetches the ZK-Proof from the chain.
    *   `helium.download_weights(job_id)`: Retrieves finalized weights after successful verification.

## 2. MINER NODE SETUP (The Compute Layer)
For hardware providers to start earning $HLM.
*   **Binary**: `helium-node`
*   **Requirements**: Linux, NVIDIA RTX 3090+ (80GB VRAM recommended), TPM 2.0 Enabled.
*   **Commands**:
    *   `helium-node init`: Registers the node on-chain.
    *   `helium-node start --gpu 0`: Joins the global job pool.

## 3. CORE PROTOCOL DOCUMENTATION
For those wanting to audit or contribute to the L1.
*   **Specs**: Detailed consensus logic for PoUW (Proof of Useful Work).
*   **Verifier Logic**: How ZK-SNARKs and Hardware Attestations are validated.
*   **P2P Network**: Discoverability and job gossip protocols (Rust/libp2p).

---

# DISCORD STRUCTURE (The Helium Forge)

## I. ENTRANCE GATE
*   `#rules-and-roles`: Verification via waitlist referral code.
*   `#announcements`: Protocol-wide updates.

## II. THE FORGE (Miners/Hardware)
*   `#gpu-optimization`: Discussions on training performance.
*   `#node-setup-help`: Technical support for `helium-node`.
*   `#mining-stats`: Bragging rights and difficulty discussions.

## III. THE LAB (AI Researchers/Devs)
*   `#sdk-support`: Helping devs submit jobs via `helium-sdk`.
*   `#model-architectures`: Discussions on PoUW-compatible models.
*   `#research-governance`: Debating the next HIPs (Helium Improvement Proposals).

## IV. GENERAL
*   `#lounge`: Philosophy of Sovereign AI.
*   `#roadmap-discussion`: Feedback on the 1-month plan.
