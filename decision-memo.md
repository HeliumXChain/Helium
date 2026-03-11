```# Decision memo

Date: 2026-03-11
Project: Helium
Status: discovery

## Current decision

- Verdict: pending (awaiting Mom Test completion)
- Geo priority: Global (decentralized P2P network)
- Target user: AI developers, ML engineers, researchers needing GPU compute for model training
- Buyer hypothesis: Startups, researchers, and companies training AI models who find AWS/Google Cloud too expensive

## Locked product definition

- Wedge: Compute-to-Earn blockchain for AI model training (Proof of Useful Work)
- In scope: 
  - Blockchain core (Rust-based)
  - P2P network for distributed compute
  - Cryptographic verification of AI training (zk-ML / PoUW)
  - Token economy for compute marketplace
- Out of scope:
  - Tor integration (removed from scope)
  - Consumer wallet app
  - Enterprise features

## Pain Point Hypothesis

**Problem**: AI model training on AWS/Google Cloud is prohibitively expensive for startups, researchers, and independent developers. GPU costs are a major barrier to AI innovation outside large tech companies.

**Solution**: Decentralized P2P network where:
- Clients pay tokens to submit AI training jobs
- Miners (GPU owners) earn tokens by providing compute power
- Uses "Proof of Useful Work" (PoUW) instead of wasteful Bitcoin mining

## Validation ladder

1. L0 - problem hypothesis (CURRENT)
2. L1 - desk evidence
3. L2 - expert confirmation
4. L3 - pilot-ready offer
5. L4 - willingness-to-pay proof

## Current evidence

- What is already proven:
  - Competitor validation: Bittensor (TAO), Gensyn, Akash Network, Render Network, Qubic, DeepBrain Chain operate in this space
  - Market demand exists for decentralized GPU compute
  - 1 Mom Test interview completed (Robino) - confirms AWS/Vast.ai usage for XXL models
  
- What is partially proven:
  - Pain point exists (high GPU costs)
  - Users currently workaround with local hardware or expensive cloud
  
- What is still unproven:
  - Willingness to use a new blockchain-based solution
  - Trust in decentralized verification
  - Complete Mom Test (need 4 more interviews)

## Competitor Landscape

| Competitor | Focus | Status |
|------------|-------|--------|
| Bittensor (TAO) | AI marketplace P2P | Active |
| Gensyn | Deep learning compute L1 | Building |
| Akash Network | Decentralized cloud | Active |
| Render Network | 3D rendering, extending to AI | Active |
| Qubic | Useful Proof of Work for NN training | Active |
| DeepBrain Chain | AI compute blockchain | Active |
| Neuromation | AI compute marketplace | Active |

## Decision rule

- Do not move to marketing or code until L1 is complete.
- Do not present willingness to pay as proven until L2 or L4 closes that gap.
- If compliance, integration, or budget owner is unclear, block outreach until expert calls close it.
- **Rule 2 BLOCKER**: Mom Test must have 5+ interviews before production code.

## Technology Stack

- **Language**: Rust (recommended for 2026+ blockchain development)
- **Architecture**: 
  - `helium/core/` - Blockchain logic (blocks, chain, transactions, consensus)
  - `helium/network/` - P2P networking, peer synchronization
  - `helium/crypto/` - Hashing, Merkle trees, signatures
  - `helium/wallet/` - Key management, transaction creation
- **Consensus**: Proof of Useful Work (PoUW) for AI training verification

## Bootstrap Feasibility

- **Phase 1 (Prototyping)**: Near-zero cost (time + discipline only)
- **Phase 2 (Testnet)**: ~100 EUR/month for VPS nodes
- **Phase 3 (Mainnet)**: Requires security audits (50k-200k USD) - fundable via VC or token sale after Phase 2 proof

## Next actions

1. [ ] Complete Mom Test: 4 more interviews needed (currently 1/5)
2. [ ] Update evidence-matrix.csv with competitor research
3. [ ] Update scorecard.md with current validation scores
4. [ ] Run `prompts/perplexity.md` for desk research
5. [ ] Run `prompts/grok.md` for public signal research
6. [ ] Create README.md with project foundation
