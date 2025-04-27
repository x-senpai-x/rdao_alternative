Eliminating Biasness from RANDAO 
===========================================================

Overview
--------

This repository documents the research, analysis, and prototype development work centered on improving randomness beacons in Ethereum's Proof-of-Stake system, with a primary focus on the Beacon Chain's RANDAO construction. The work aims to deeply understand existing designs, identify vulnerabilities, and explore robust cryptographic improvements with practical prototypes.

Research Focus
--------------

-   **Understanding RANDAO**: Detailed exploration of the current RANDAO-based randomness mechanism in Ethereum 2.0:

    -   Accumulation of randomness through BLS signature-based reveals.

    -   XOR-based mixing of hashed BLS signatures.

    -   Epoch-wise committee and proposer selection seeded by RANDAO outputs.

-   **Cryptographic Foundations**:

    -   Use of **BLS Signatures** for verifiable, unique contributions.

    -   Hash functions (SHA-256) for signature compression and uniformity.

    -   XOR mixing properties (associativity and commutativity) to limit attack surface.

-   **Identified Vulnerabilities**:

    -   Selfish Mixing / Block Withholding Attack

    -   Last Revealer Attack

    -   Forking Attacks (manipulating beacon state through selective block production)

Each vulnerability was formally studied, leveraging academic papers, Ethereum Research posts, and in-depth Ethereum consensus specifications.

Implementation Progress
-----------------------

### 1\. Basic RANDAO Prototype (Rust)

-   **Goal**: Implement the core RANDAO update cycle mimicking Ethereum's Beacon Chain process.

-   **Tech Stack**:

    -   `threshold_bls` crate for BLS signature generation and verification.

    -   `sha2` crate for SHA-256 hashing.

-   **Core Steps Implemented**:

    -   BLS Key generation per validator.

    -   Signing the current epoch using private BLS key.

    -   Hashing the signature output.

    -   XOR mixing into the global `randao_mix` accumulator.

Code location: 
### 2\. Threshold BLS Signing Prototype (Rust)

-   **Goal**: Demonstrate threshold signing as a potential unbiasable randomness source.

-   **Tech Stack**:

    -   `threshold_bls` crate for polynomial secret sharing and threshold aggregation.

-   **Core Steps Implemented**:

    -   Distributed Key Generation (DKG) setup: Splitting the BLS secret key into shares.

    -   Each validator produces a partial signature for the message (epoch data).

    -   Aggregating threshold partials to recover the full BLS signature.

    -   Verifying the aggregated signature against the group public key.

Code location: 

Proposed Solution
-----------------

### Threshold BLS-Based Random Beacon with Fallback and VDF

**Design Components**:

-   **Threshold BLS Signatures**:

    -   Validators collaboratively produce a threshold BLS signature on the epoch seed.

    -   The threshold signature output is deterministic and unique once enough shares are combined, eliminating the "last-revealer" bias.

-   **RANDAO Fallback**:

    -   If threshold signing fails due to offline validators or network failures, fall back to standard RANDAO XOR-based accumulation to maintain liveness.

-   **Optional VDF Post-Processing**:

    -   After collecting the threshold signature (or RANDAO fallback mix), run it through a Verifiable Delay Function (VDF) to add delay-based unpredictability.

**Cryptographic Assumptions**:

-   Hardness of the discrete logarithm problem over elliptic curves (BLS signatures).

-   Correctness and unforgeability of polynomial secret sharing (threshold schemes).

-   Non-parallelizability and soundness of the VDF computation.

**Security Goals**:

-   Eliminate biasability (adaptive or selfish mixing) in beacon randomness.

-   Guarantee liveness through fallback mechanisms.

-   Achieve cryptographic unpredictability with minimal additional complexity.

**Advantages of Chosen Approach**:

-   Strong bias resistance even under adversarial stake conditions.

-   Seamless fallback to current RANDAO under partial validator downtime.

-   Clear upgrade path compatible with Ethereum's existing validator infrastructure.

-   Extensible to future improvements (e.g., Distributed Validator Technology (DVT)).

Current Progress Summary
------------------------

-   Wrote a blog summarizing my study [here](https://0xsenpai.substack.com/p/understanding-ethereums-randao-mechanism)

-   Completed detailed literature survey and formal analysis of Ethereum's current RANDAO mechanism and its cryptographic basis.

-   Implemented basic RANDAO prototype mimicking Ethereum's production system.

-   Implemented threshold BLS signing prototype demonstrating share generation, aggregation, and verification.

-   Finalized a clear, detailed proposal favoring a hybrid threshold-RANDAO-VDF beacon design.

-   Documented cryptographic assumptions, adversarial model, and technical workflow for proposed design.

Existing VRFs
------------------------
| **Beacon / Approach** | **Strengths** | **Weaknesses** |
| --- | --- | --- |
| **DFINITY (Internet Computer)**
*Threshold BLS beacon*​[near.org](https://near.org/blog/randomness-in-blockchain-protocols) | Produces truly *unbiased, unpredictable* outputs via threshold BLS (acting as a VRF). Well-studied in production design. It achieves a single collective signature per round. | Requires expensive setup and coordination (large n-of-k DKG across nodes)​[a16zcrypto.com](https://a16zcrypto.com/posts/article/public-randomness-and-randomness-beacons/#:~:text=A%20downside%20of%20these%20approaches,the%20protocols%20are%20very%20efficient). The fixed committee model incurs latency and complexity. Without an additional VDF layer, it depends on at least one honest node for unpredictability. |
| **NEAR Protocol** 
(threshold-BLS beacon)​[near.org](https://near.org/blog/randomness-threshold-signatures#:~:text=Back%20in%202015%2C%20DFinity%20made,is%20both%20unbiased%20and%20unpredictable) | Similar to DFINITY (modeled off it). Uses threshold BLS for unbiased randomness. Deeply researched by NEAR/DFINITY teams. | Same tradeoffs as DFINITY: complex DKG/resharing needed, limited resiliency when keys reconfigure. As of 2020, live deployments of full beacon were rare. |
| **Drand / League of Entropy** 
*Public randomness service*​[developers.cloudflare.com](https://developers.cloudflare.com/randomness-beacon/cryptographic-background/randomness-generation/#:~:text=BLS%20Signatures) | Public, open-source beacon run by a mixed group (Cloudflare, universities). Uses threshold BLS signatures on a counter​. Outputs are *unbiasable* (deterministic BLS sig)​. Well-tested (30s rounds). | Relies on a *trusted set of operators*. Initial key must be securely generated and requires updates if nodes change. If enough operators' keys are compromised, the output becomes predictable​. Not integrated into Ethereum (external service). |
| **Chainlink VRF** 
*Oracle-based VRF*​[a16zcrypto.com](https://a16zcrypto.com/posts/article/public-randomness-and-randomness-beacons/#:~:text=Chainlink%20VRF%20combines%20this%20approach,or%20thresholded%20to%20a%20group) | Provides verifiable randomness **on-demand** to smart contracts. Easy to use in current Ethereum DApps. Combines VRF with an oracle network, achieving cryptographic proofs of correctness​. | Not a public beacon in itself; serves individual requests. Depends on Chainlink's oracle set, so involves trust in those operators. Transactional gas costs per request. Has limited scalability for large-scale beacon use. |
| **Ethereum RANDAO (current)** 
*On-chain XOR-commit*​[arxiv.org](https://arxiv.org/pdf/2409.19883#:~:text=Proof%20of%20Stake%20Ethereum%20provides,to%20the%20public%2C%20it%20is) | Built into Ethereum's consensus (Beacon Chain and EIP-4788). No off-chain trust required. Relatively simple: each proposer's BLS signature is XORed into a running seed​. Fast and low-overhead. | **Biasable**: a proposer withholding oracles can *maximize* next-epoch advantage​. Known "Last Revealer" attacks (tail-of-epoch withholding) significantly bias the outcome​  Without a VDF or threshold aggregation, outputs can be predicted by colluding validators. |

Next Steps
----------

-   Expand threshold signature prototype to simulate real validator network behavior (message gossipping, timeouts).

-   Implement VDF prototype integration (e.g., simulate VDF computation delay on threshold outputs).

-   Analyze performance implications of the threshold and fallback transitions.

-   Explore dynamic DKG refresh protocols for rotating validator sets.

-   Formalize specifications and security proofs for the proposed beacon design.

* * * * *

This repository serves as a live record of research-driven protocol development work towards improving the robustness of randomness beacons in Proof-of-Stake blockchains.
