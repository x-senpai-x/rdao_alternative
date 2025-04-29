use blst::min_pk::{PublicKey, Signature};
use sha2::{Sha256, Digest};
use crate::constants::{EPOCHS_PER_HISTORICAL_VECTOR, MIN_SEED_LOOKAHEAD, DOMAIN_BEACON_PROPOSER, DOMAIN_BEACON_ATTESTER, DOMAIN_RANDAO, DST};

/// The on-chain beacon state (only the RANDAO part shown)
pub struct BeaconState {
    pub randao_mixes: [[u8; 32]; EPOCHS_PER_HISTORICAL_VECTOR],
}

impl BeaconState {
    pub fn new() -> Self {
        Self { randao_mixes: [[0u8;32]; EPOCHS_PER_HISTORICAL_VECTOR] }
    }

    // Process a block’s randao_reveal:
    // verify BLS sig over (epoch || DOMAIN_RANDAO)
    // hash it
    // XOR into the state’s mix for this epoch
    pub fn process_randao(
        &mut self,
        reveal: &Signature,
        pk: &PublicKey,
        epoch: u64,
    ) -> Result<(), String> {
        let epoch_bytes = epoch.to_le_bytes();
        // verify
        let err = reveal.verify(true, &epoch_bytes, DST, &[], pk, true);
        if err != blst::BLST_ERROR::BLST_SUCCESS {
            return Err(format!("Invalid randao_reveal: {:?}", err));
        }
        // hash signature
        let mut hasher = Sha256::new();
        hasher.update(&reveal.to_bytes());
        let h = hasher.finalize();
        // XOR into mix
        let idx: usize = (epoch as usize) % EPOCHS_PER_HISTORICAL_VECTOR;//epoch stores the actual epoch no but idx mods it to lookback period
        for i in 0..32 {
            self.randao_mixes[idx][i] ^= h[i]; //for all slots
        }
        Ok(())
    }

    /// Derive a 32-byte seed for use in epoch `target_epoch` from mix at `target_epoch - lookahead - 1`
    pub fn get_seed(&self, target_epoch: u64, domain: [u8;4]) -> [u8;32] {
        let mix_epoch = target_epoch.checked_sub(MIN_SEED_LOOKAHEAD + 1).unwrap_or(0) as usize % EPOCHS_PER_HISTORICAL_VECTOR;
        //calculates the epoch from which randomness is to be derived
        let mix = &self.randao_mixes[mix_epoch];
        let mut hasher = Sha256::new();
        hasher.update(&domain);
        hasher.update(&target_epoch.to_le_bytes());
        hasher.update(mix);
        let out = hasher.finalize();
        let mut seed = [0u8;32];
        seed.copy_from_slice(&out);
        seed
    }
}

/// Deterministic “shuffle” of `n` items given a seed — here we do a simple
/// Fisher-Yates with seed bytes as randomness.
pub fn shuffle_indices(n: usize, seed: &[u8;32]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..n).collect();
    let mut rnd = seed.clone(); // mutable copy
    for i in (1..n).rev() {
        // use first 8 bytes of rnd for a u64, mod (i+1)
        let mut b = [0u8;8];
        b.copy_from_slice(&rnd[0..8]);
        let r = u64::from_le_bytes(b) as usize % (i+1);
        idx.swap(i, r);
        // rotate rnd for next iteration
        rnd.rotate_left(1);
    }
    idx
}

/// Choose the block proposer for a slot (selector) given `validator_count` and a seed.
pub fn select_proposer(validator_count: usize, seed: &[u8;32]) -> usize {
    // shuffle all validator indices
    let shuffled = shuffle_indices(validator_count, seed);
    // pick the first one
    shuffled[0]
}
