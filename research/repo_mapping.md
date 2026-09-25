# HELIUM HYBRID REPOSITORY MAPPING

To protect Helium's IP while maximizing developer adoption, we will use the following structure:

## 1. PUBLIC REPOSITORIES (Visible to all)
*   **[Helium](https://github.com/Lemniscate-world/Helium)**: The "Main" public entry point.
    *   **Contents**: README, Whitepaper, Developer Guide, License.
    *   **Purpose**: Strategic visibility and trust building.
*   **`helium-sdk`**:
    *   **Contents**: Python and Rust libraries for job submission.
    *   **Purpose**: Onboarding AI researchers and developers.
*   **`helium-node-cli`**:
    *   **Contents**: Terminal interface for miners to start nodes.
    *   **Purpose**: Onboarding hardware providers.

## 2. PRIVATE REPOSITORIES (Internal only)
*   **`helium-core-engine`**:
    *   **Contents**: P2P networking, PoUW consensus logic, and blockchain state transition.
*   **`helium-zk-verifier`**:
    *   **Contents**: ZK-SNARK circuits and hardware attestation verification logic.

## 3. LANDING PAGE UPDATE
The landing page links should point to:
*   **GitHub**: `https://github.com/Lemniscate-world/Helium` (The public hub).
*   **Discord**: `https://discord.gg/[INVITE_CODE]` (Waitlist exclusive).

## 4. NEXT STEPS
1.  Initialize `Helium` public repo with the current research docs.
2.  Update `app/page.tsx` on the landing page with the correct GitHub link.
3.  Set up the Discord server following "The Helium Forge" structure.
