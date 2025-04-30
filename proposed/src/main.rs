// src/main.rs
mod threshold;
use threshold;
use threshold_bls::{sig::Scheme as _, poly::Poly, poly::Idx, sig::Share};
use rand::thread_rng;
use blst::min_pk::{SecretKey, PublicKey, Signature};
use beacon_randofix::{
    BeaconState, shuffle_indices, select_proposer,
    DOMAIN_BEACON_PROPOSER, DOMAIN_BEACON_ATTESTER, DOMAIN_RANDOMNESS, DST,
};

fn main() {
    // === DKG (off-chain) ===
    let n = 10;      // committee size
    let t = 7;       // threshold
    let mut rng = thread_rng();

    // 1. Each validator generates a secret polynomial:
    let secret_poly = Poly::<SecretKey>::new(t - 1);
    // 2. Commit to coefficients & distribute shares:
    let shares: Vec<Share> = (0..n)
        .map(|i| {
            let eval = secret_poly.eval(i as Idx);
            Share { index: eval.index, private: eval.value }
        })
        .collect();
    // 3. Public commitments and group public key:
    let pub_poly = secret_poly.commit();
    let group_pk = pub_poly.public_key();

    // === Beacon State ===
    let mut state = BeaconState::new(group_pk.clone());

    // Simulate end of epoch N = 1000
    let epoch = 1_000u64;

    // Each validator (i < t) produces a partial signature on `epoch`:
    let partials: Vec<_> = shares.iter()
        .map(|share| SigScheme::partial_sign(share, &epoch.to_le_bytes()).unwrap())
        .collect();

    // Once we have ≥ t partials, aggregate into Σ_N:
    let sigma = SigScheme::aggregate(t, &partials[..t]).unwrap();
    let sigma_sig = Signature::from_bytes(&sigma.as_bytes()).unwrap();

    // Process the threshold signature (is_threshold = true)
    state.process_reveal(epoch, &sigma_sig, true)
        .expect("threshold signature must verify");

    // Derive seeds for epoch N+2
    let target = epoch + 2;
    let prop_seed = state.get_seed(target, DOMAIN_BEACON_PROPOSER);
    let att_seed  = state.get_seed(target, DOMAIN_BEACON_ATTESTER);
    let ran_seed  = state.get_seed(target, DOMAIN_RANDOMNESS);

    // Use existing utilities:
    let proposer = select_proposer(16_384, &prop_seed);
    println!("Epoch {} proposer for slot 0 is validator #{}", target, proposer);

    // … and so on for committees, attesters, etc.
}
