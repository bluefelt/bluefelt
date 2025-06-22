//! Random number generation for game lobbies

use parking_lot::Mutex;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use sha2::{Sha256, Digest};

/// Wrapper for deterministic random number generation in games
pub struct GameRng {
    rng_state: Mutex<RngState>,
    seed: [u8; 32],
}

struct RngState {
    rng: StdRng,
    position: u64,
}

impl GameRng {
    /// Create a new GameRng with optional seed
    pub fn new(seed: Option<[u8; 32]>) -> Self {
        let seed = seed.unwrap_or_else(|| {
            let mut hasher = Sha256::new();
            hasher.update(uuid::Uuid::new_v4().to_string().as_bytes());
            let result = hasher.finalize();
            let mut seed_bytes = [0u8; 32];
            seed_bytes.copy_from_slice(&result);
            seed_bytes
        });
        
        let rng = StdRng::from_seed(seed);
        
        Self {
            rng_state: Mutex::new(RngState {
                rng,
                position: 0,
            }),
            seed,
        }
    }
    
    /// Get the next random u32 value
    pub fn next_u32(&self) -> u32 {
        let mut state = self.rng_state.lock();
        state.position += 1;
        state.rng.gen()
    }
    
    /// Get the next random value in range [min, max)
    pub fn next_range(&self, min: u32, max: u32) -> u32 {
        let mut state = self.rng_state.lock();
        state.position += 1;
        state.rng.gen_range(min..max)
    }
    
    /// Get current RNG position for debugging
    pub fn get_position(&self) -> u64 {
        let state = self.rng_state.lock();
        state.position
    }
    
    /// Get the RNG seed as hex string for debugging/display
    pub fn get_seed_hex(&self) -> String {
        hex::encode(&self.seed)
    }
    
    /// Create a new StdRng instance with the same seed
    pub fn create_rng(&self) -> StdRng {
        StdRng::from_seed(self.seed)
    }
}