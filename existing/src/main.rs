mod randao;
mod constants;
use crate::randao::{BeaconState};
use blst::min_pk::{SecretKey, PublicKey, Signature};
use rand::RngCore;
use crate::constants::{EPOCHS_PER_HISTORICAL_VECTOR, MIN_SEED_LOOKAHEAD, DOMAIN_BEACON_PROPOSER, DOMAIN_BEACON_ATTESTER, DOMAIN_RANDAO, DST};

fn main() {
    // Setup a validator’s BLS keypair
    let mut rng = rand::rng();
    let mut ikm = [0u8;32];
    rng.fill_bytes(&mut ikm);
    let sk = SecretKey::key_gen(&ikm, &[]).unwrap();
    let pk = sk.sk_to_pk(); // Example method, replace with the actual method from `blst`
    
    // Initialize beacon state
    let mut state = BeaconState::new();

    // Suppose we are in epoch 1000
    let epoch = 1000u64;

    // Proposer signs the current epoch number to produce randao_reveal
    let reveal = sk.sign(&epoch.to_le_bytes(), DST, &[]);

    // Process the randao_reveal into state
    state.process_randao(&reveal, &pk, epoch).expect("randao processing failed");//updates the randao_mixes

    // Now derive seeds for epoch+2 duties
    let target_epoch = epoch + 2;
    let proposer_seed = state.get_seed(target_epoch, DOMAIN_BEACON_PROPOSER);
    let attester_seed = state.get_seed(target_epoch, DOMAIN_BEACON_ATTESTER);
    let randao_seed   = state.get_seed(target_epoch, DOMAIN_RANDAO);

    // Select block proposer (out of assume 16_384 validators ).16,384(32 × 512 = 16,384) is the target number of active validators per epoch in Ethereum's beacon chain
    //this is done once for each slot
    let validator_count=16_384; //state.validators
    let proposer_index = randao::select_proposer(validator_count, &proposer_seed);
    println!("Slot proposer for epoch {} will be validator #{}", target_epoch, proposer_index);

    // The first 512 indices from the shuffled list are selected as the sync committee.    
    //this is done once every 256 epochs,
    let sync_committee = randao::shuffle_indices(validator_count, &randao_seed)[..512].to_owned();
    println!("Sync committee validators: {:?}", &sync_committee[..8]); // first 8

    //This is done once for each epoch
    let attester_indices = randao::shuffle_indices(validator_count, &attester_seed);
    let committees_per_slot = 64; //upto 64 attessation committes per slot
    let committee_size = validator_count/(committees_per_slot*32); // depends on total validators
    let total_committes=committees_per_slot*32;
    for i in 0..total_committes {
        let slot=i/committees_per_slot;//which slot
        let committee=i%committees_per_slot;//which committee
        let start    = i * committee_size;
        let end = start+committee_size;
        let validators_in_committee = &attester_indices[start..end];
        println!("Epoch {} slot {} committee #{:?} = {:?}",target_epoch, slot, committee, validators_in_committee);
    }
}

