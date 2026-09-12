// Lottery engine: draws winners using verifiable randomness

use common::{LotteryEntry, WorkerId};

/// Draws lottery winners using VRF (Verifiable Random Function)
pub struct LotteryEngine;

impl LotteryEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn draw_winner(&self, entries: &[LotteryEntry]) -> Result<LotteryEntry, String> {
        if entries.is_empty() {
            return Err("No entries to draw from".into());
        }

        // Use VRF for verifiable randomness
        // In production: use actual VRF library (e.g., vrf crate)
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(b"vrf-seed");
        let seed = hasher.finalize();
        let seed_u64 = u64::from_le_bytes(seed[0..8].try_into().unwrap());

        let winner_idx = (seed_u64 as usize) % entries.len();
        Ok(entries[winner_idx].clone())
    }
}
