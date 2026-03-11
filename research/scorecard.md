I need to update the scorecard.md with current validation scores based on the evidence gathered. Let me write a comprehensive scorecard based on the research and Mom Test results.```# Scorecard

Date: 2026-03-11
Project: Helium
Current validation stage: L0
Scale: 1 to 5
Weighted points formula: `weight * score / 5`

## Fixed axes

| Axis | Weight | Score | What is proven | What is still missing |
| --- | ---: | ---: | --- | --- |
| Urgence | 20 | 2 | AWS/Vast.ai usage confirms GPU cost pain point (Mom Test #1), multiple competitors validate market demand | Need 4 more Mom Test interviews to confirm urgency pattern |
| Budget signal | 20 | 1 | Competitor market exists (Bittensor, Gensyn, Akash) but no direct buyer interview yet | Need expert calls to confirm budget owner and procurement path |
| Speed to first contract | 15 | 1 | Technical stack defined (Rust, PoUW) but no pilot offer defined | Need L2 expert confirmation on integration path |
| Regulatory friction | 15 | 2 | Blockchain regulatory environment varies by geo, crypto tokens face scrutiny | Need legal analysis for token classification |
| Integration load | 10 | 1 | Users currently use Docker + cloud (AWS/Vast.ai) — suggests integration complexity | Need to understand current workflow in detail |
| Platform dependency | 10 | 2 | Decentralized by design — no single platform dependency, but relies on GPU hardware availability | Need to assess GPU miner onboarding complexity |
| Competitive intensity | 5 | 3 | Bittensor, Gensyn, Akash, Render, Qubic, DeepBrain Chain all active — high competition | Need differentiation strategy |
| Defensability | 5 | 1 | Open-source blockchain — low moat unless network effects kick in fast | Need to identify unique value proposition |

## Weighted Score Calculation

| Axis | Weight | Score | Weighted Points |
| --- | ---: | ---: | ---: |
| Urgence | 20 | 2 | 8.0 |
| Budget signal | 20 | 1 | 4.0 |
| Speed to first contract | 15 | 1 | 3.0 |
| Regulatory friction | 15 | 2 | 6.0 |
| Integration load | 10 | 1 | 2.0 |
| Platform dependency | 10 | 2 | 4.0 |
| Competitive intensity | 5 | 3 | 3.0 |
| Defensability | 5 | 1 | 1.0 |
| **TOTAL** | **100** | | **31.0 / 100** |

## Stage gates

- L0 -> L1: at least 5 recent dated signals and no critical safety contradiction.
  - **Status**: IN PROGRESS — 6 signals collected (competitor analysis + Mom Test #1)
  - **Blocker**: Need 4 more Mom Test interviews (currently 1/5)
  
- L1 -> L2: one credible buyer hypothesis and one credible integration hypothesis.
  - **Status**: NOT STARTED
  
- L2 -> L3: expert calls confirm buyer, approval path, and low-friction pilot scope.
  - **Status**: NOT STARTED
  
- L3 -> L4: one clear commercial ask and one measurable pilot KPI.
  - **Status**: NOT STARTED

## Decision rule

- Do not move to marketing or code if any of these axes is still `1/5`: regulatory friction, integration load, or platform dependency.
  - **Current Status**: 
    - Regulatory friction: 2/5 (acceptable)
    - Integration load: 1/5 (BLOCKER)
    - Platform dependency: 2/5 (acceptable)
  - **Blocker**: Integration load at 1/5 — need to understand user workflow complexity

- Do not claim willingness to pay from desk research alone.
  - **Status**: No willingness to pay claimed yet — awaiting Mom Test completion

## Evidence Summary

### Positive Signals
- Market exists: 7+ competitors active in decentralized AI compute space
- Pain point validated: Mom Test #1 confirms AWS/Vast.ai usage for XXL models
- Technology feasible: Rust + PoUW approach documented, competitors implementing similar solutions
- Bootstrap affordable: Phase 1 (prototyping) near-zero cost

### Negative Signals
- High competition: Bittensor, Gensyn, Akash already have market presence
- Low defensibility: Open-source blockchain has limited moat
- Integration unknown: Need to understand Docker/cloud workflow complexity

### Missing Evidence
- 4 more Mom Test interviews needed
- Budget owner identification
- Integration workflow mapping
- Token regulatory classification
- Differentiation strategy vs competitors

## Next Actions

| Priority | Action | Blocks |
| --- | --- | --- |
| 1 | Complete Mom Test (4 more interviews) | L0 -> L1 |
| 2 | Map current integration workflow (Docker/cloud) | Integration load axis |
| 3 | Identify budget owner profile | Budget signal axis |
| 4 | Run perplexity.md for regulatory analysis | Regulatory friction axis |
| 5 | Define differentiation strategy | Defensability axis |
