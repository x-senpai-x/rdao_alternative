/// Number of epochs we keep history for (2^16 in Ethereum spec)
pub const EPOCHS_PER_HISTORICAL_VECTOR: usize = 1 << 16;
/// Lookahead for seed derivation (2 epochs in Phase0)
pub const MIN_SEED_LOOKAHEAD: u64 = 1;

/// Domain bytes for different uses (little-endian 4-byte)
pub const DOMAIN_BEACON_PROPOSER: [u8;4] = [0x00,0x00,0x00,0x00];
pub const DOMAIN_BEACON_ATTESTER: [u8;4]=[0x01,0x00,0x00,0x00];
pub const DOMAIN_RANDAO: [u8;4]         =[0x02,0x00,0x00,0x00];
pub const DST: &[u8]                    = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";//Domain separation tag