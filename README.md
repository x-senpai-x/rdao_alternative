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


Proposed Solution
-----------------

### Threshold BLS Beacon with One-Epoch Lookahead

**DKG Key Setup** Validators run an off-chain DKG to generate a shared BLS group key and secret shares.

**Epoch‑End Signing**

During epoch N, each threshold committee member produces a partial BLS signature on a domain-separated message for epoch N + 1.

Gossip and aggregate ≥ t shares into a single threshold signature Σₙ.

**On‑Chain Integration**

Add a next_epoch_randao field or replace the existing randao_mix at epoch boundary with Σₙ.

Verify Σₙ under the group public key (one extra BLS verify per epoch).

**One‑Epoch Delay**

Committees and proposers for epoch N + 1 use Σₙ directly.

Eliminates same-epoch bias and reduces lookahead from 2→1 epochs.

**Fallback Mechanism**

If threshold signing fails by epoch end, revert to legacy RANDAO XOR mix for that epoch to preserve liveness.


-   **Future Plan: VDF Post-Processing**:

    -   After collecting the threshold signature (or RANDAO fallback mix), run it through a Verifiable Delay Function (VDF) to add delay-based unpredictability.
    - 

**Cryptographic Assumptions**:

-   BLS Signature Security: Unforgeability under the XDH assumption.

-   Shamir Secret Sharing: Correctness and unforgeability of polynomial secret sharing (threshold schemes).



**Security Goals**:

Eliminate Last-Revealer Bias: No single validator (or coalition < t) can skew randomness.

Preserve Liveness: Fall back to RANDAO if threshold output is unavailable.

Minimize On‑Chain Overhead: Single BLS verify + 48‑byte signature per epoch.

Maintain Compatibility: Leverage existing BLS primitives and seed lookahead logic.

**Advantages of Chosen Approach**:

-   Strong bias resistance even under adversarial stake conditions.

-   Seamless fallback to current RANDAO under partial validator downtime.

-   Clear Upgrade Path—consensus changes are local to BLS and seed logic.

-   Extensible to future improvements (e.g., Distributed Validator Technology (DVT)).


Implementation Progress
-----------------------

### 1\. Basic RANDAO Prototype (Rust)

-   **Goal**: Implement the core RANDAO update cycle mimicking Ethereum's Beacon Chain process.

-   **Tech Stack**:

    -   `bls` crate for BLS signature generation and verification.

    -   `sha2` crate for SHA-256 hashing.

-   **Core Steps Implemented**:

    -   BLS Key generation per validator.

    -   Signing the current epoch using private BLS key.

    -   Hashing the signature output.

    -   XOR mixing into the global `randao_mix` accumulator.

Code location: existing/src
### 2\. Threshold BLS Signing Prototype (Rust)

-   **Goal**: Demonstrate threshold signing as a potential unbiasable randomness source.

-   **Tech Stack**:

    -   `threshold_bls` crate for polynomial secret sharing and threshold aggregation.

-   **Core Steps Implemented**:

    -   Distributed Key Generation (DKG) setup: Splitting the BLS secret key into shares.

    -   Each validator produces a partial signature for the message (epoch data).

    -   Aggregating threshold partials to recover the full BLS signature.

    -   Verifying the aggregated signature against the group public key.

Code location: proposed/src

Alternative Solutions looked at 
-----------------

Threshold Encryption(commitment + encryption + proof)
-----------------
 Here, validators jointly generate a threshold public key (via a distributed key generation) for a homomorphic encryption scheme (threshold encryption typically uses either ElGamal (elliptic-curve DDH) or Paillier (composite residuosity/factoring) as its homomorphic scheme). On each slot, the proposer encrypts a fresh random share under this public key and includes the ciphertext in their block. All ciphertexts from the epoch are homomorphically combined (for example, by multiplying all ElGamal ciphertexts together) into one aggregate ciphertext representing the sum of all secret shares. At epoch’s end, a threshold subset of validators cooperatively decrypt this aggregate ciphertext, recovering the combined randomness. Because each share was encrypted and sealed at proposal time, no single proposer can see or adapt to others’ shares, thus preventing the usual last-revealer bias. If for any reason threshold decryption fails (e.g. insufficient decryptors), the scheme falls back to Ethereum’s existing RANDAO mix for that epoch. Every block proposal would carry a large ciphertext and proof, and the consensus protocol would have to track this state. In short, threshold encryption adds significant protocol complexity, whereas threshold-BLS+VDF mainly adds the (also complex) VDF step but reuses the existing signature infrastructure.
 Storage and Bandwidth Overhead:
Under threshold encryption, each block (slot) must carry an encryption share. For example, an ElGamal ciphertext on a 256-bit curve is two group elements (roughly 64–128 bytes) plus any proof. If there are 32 slots per epoch, that’s on the order of 4–8 KB per epoch just for ciphertext (not counting proofs), versus ~3 KB per epoch today (32 blocks × 96-byte BLS sig). Aggregation can be done off-chain or on-chain, but if done on-chain it consumes additional gas. Conversely, a threshold-BLS scheme would have each signer produce a small signature (96 bytes) and the aggregator only publishes one combined signature per epoch or block. In practice, threshold encryption clearly costs more bandwidth/storage per slot than a single BLS signature. Even without VDF, the on-chain payload under encryption-based beacon is much larger and more costly to verify (pairing vs. simple EC exponentiation). (For completeness: VDF outputs also have a size and proof cost, but these are expected to be only a few hundred bytes and are submitted by one node per epoch.)
The threshold-encryption idea effectively trades one problem (last-proposer bias) for a host of new issues (complex DKG, proofs, larger blocks) with no clear safety advantage once fully analyzed.


Current Progress Summary
------------------------

-   Wrote a blog summarizing my study [here](https://0xsenpai.substack.com/p/understanding-ethereums-randao-mechanism)

-   Completed detailed literature survey and formal analysis of Ethereum's current RANDAO mechanism and its cryptographic basis.

-   Implemented basic RANDAO prototype mimicking Ethereum's production system.

-   Implemented threshold BLS signing prototype demonstrating share generation, aggregation, and verification.

-   Comprehensive protocol specification drafted.

Existing Research
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

Notes: DFINITY/NEAR and Drand rely on threshold BLS signatures, giving strong unbiasability under honest-majority assumptions​. However, all threshold schemes face key-management complexity​. Chainlink VRF’s oracle model sidesteps these but at the cost of trust and cost overhead​. Ethereum’s on-chain RANDAO is simple and trustless but suffers from strategic bias if multiple late-slot validators collude. 
The proposed RANDAO+threshold BLS+VDF design aims to combine these benefits: it keeps the on-chain nature of RANDAO while using a group signature (threshold BLS) to prevent a single proposer from controlling the seed, and applies a VDF to eliminate last-revealer bias. This design thus strives to achieve *unbiasability* and *unpredictability* similar to DFINITY/Drand, without requiring a pre-established oracle service.

Next Steps
----------

-   Expand threshold signature prototype to simulate real validator network behavior (message gossipping, timeouts).


-   Analyze performance implications of the threshold and fallback transitions.

-   Explore dynamic DKG refresh protocols for rotating validator sets.

-   Formalize specifications and security proofs for the proposed beacon design.

* * * * *

This repository serves as a live record of research-driven protocol development work towards improving the robustness of randomness beacons in Proof-of-Stake blockchains.
