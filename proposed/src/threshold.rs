// src/lib.rs
// Proposed Threshold BLS + One‐Epoch Lookahead Beacon Prototype

use blst::min_pk::{PublicKey, Signature};
use sha2::{Digest, Sha256};
use threshold_bls::{
    schemes::bls12_381::G1Scheme as SigScheme,   // threshold‐BLS scheme
    poly::Poly, poly::Idx, sig::Share,            // DKG and share types
    sig::Scheme as _,                            // for sign/verify/aggregate
};
use rand::thread_rng;
use crate::constants::{
    EPOCHS_PER_HISTORICAL_VECTOR, MIN_SEED_LOOKAHEAD,
    DOMAIN_BEACON_PROPOSER, DOMAIN_BEACON_ATTESTER,
    DOMAIN_RANDOMNESS, DST,
};

// BeaconState now holds both RANDAO fallback mixes and the group public key
// plus the latest threshold signature for each epoch.
pub struct BeaconState {
    pub randao_mixes: [[u8; 32]; EPOCHS_PER_HISTORICAL_VECTOR],
    pub threshold_pk: PublicKey,               // BLS group public key
    pub threshold_sigs: Vec<Option<Signature>>, // Σ_N for each epoch 
}

impl BeaconState {
    /// Initialize with a given group public key.
    pub fn new(group_pk: PublicKey) -> Self {
        Self {
            randao_mixes: [[0u8; 32]; EPOCHS_PER_HISTORICAL_VECTOR],
            threshold_pk: group_pk,
            threshold_sigs: vec![None; EPOCHS_PER_HISTORICAL_VECTOR],
        }
    }

    /// Process either a partial signature share or fallback RANDAO reveal:
    /// - If `is_threshold` is true, `sig` is interpreted as the **final** Σ_N
    ///   and stored for epoch `n`.
    /// - Otherwise, we hash-and-XOR it into `randao_mixes` as before.
    pub fn process_reveal(
        &mut self,
        epoch: u64,
        sig: &Signature,
        is_threshold: bool,
    ) -> Result<(), String> {
        let idx = (epoch as usize) % EPOCHS_PER_HISTORICAL_VECTOR;
        if is_threshold {
            // Verify the *group* signature Σ_n on message (epoch || DST)
            let msg = epoch.to_le_bytes();
            let err = sig.verify(true, &msg, DST, &[], &self.threshold_pk, true);
            if err != blst::BLST_ERROR::BLST_SUCCESS {
                return Err(format!("Invalid threshold Σ for epoch {}: {:?}", epoch, err));
            }
            self.threshold_sigs[idx] = Some(sig.clone());
        } else {
            // Fallback: treat as legacy RANDAO reveal, same as before
            let msg = epoch.to_le_bytes();
            let err = sig.verify(true, &msg, DST, &[], &self.threshold_pk, true);
            if err != blst::BLST_ERROR::BLST_SUCCESS {
                return Err(format!("Invalid randao_reveal: {:?}", err));
            }
            let mut hasher = Sha256::new();
            hasher.update(&sig.to_bytes());
            let h = hasher.finalize();
            for i in 0..32 {
                self.randao_mixes[idx][i] ^= h[i];
            }
        }
        Ok(())
    }

    /// Derive a 32-byte seed for epoch `target_epoch` by looking back
    /// one-epoch lookahead on *threshold* signature if present,
    /// else fallback to RANDAO mix.
    pub fn get_seed(&self, target_epoch: u64, domain: [u8;4]) -> [u8;32] {
        let mix_epoch = target_epoch
            .checked_sub(MIN_SEED_LOOKAHEAD + 1)
            .unwrap_or(0) as usize % EPOCHS_PER_HISTORICAL_VECTOR;

        // Prefer threshold signature Σ_{mix_epoch} if available:
        let raw = if let Some(ref sigma) = self.threshold_sigs[mix_epoch] {
            // Optional lightweight delay: hash-chain or single SHA256
            let mut h = sha2_chain(&sigma.to_bytes(), 1); // e.g. 1 extra hash
            h.to_vec()
        } else {
            // Fallback: use legacy randao mix
            self.randao_mixes[mix_epoch].to_vec()
        };

        // Final domain‐separated seed
        let mut hasher = Sha256::new();
        hasher.update(&domain);
        hasher.update(&target_epoch.to_le_bytes());
        hasher.update(&raw);
        let out = hasher.finalize();
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&out);
        seed
    }
}

/// Simple helper: apply `n` rounds of SHA-256 to input
fn sha2_chain(mut data: &[u8], rounds: usize) -> [u8; 32] {
    let mut h = [0u8; 32];
    let mut hasher = Sha256::new();
    for _ in 0..rounds {
        hasher.update(data);
        let digest = hasher.finalize_reset();
        data = &digest;
        h.copy_from_slice(&digest);
    }
    h
}

/// The same shuffle/selection utilities as before
pub fn shuffle_indices(n: usize, seed: &[u8;32]) -> Vec<usize> { /* … */ }
pub fn select_proposer(validator_count: usize, seed: &[u8;32]) -> usize { /* … */ }

