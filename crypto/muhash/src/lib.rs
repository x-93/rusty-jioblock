//! MuHash accumulator for UTXO set commitments

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Default, Clone)]
pub struct MuHash {
    state: u64,
}

pub const EMPTY_MUHASH: MuHash = MuHash { state: 1 };

impl MuHash {
    pub fn new() -> Self { Self { state: 1 } }
    pub fn add<T: Hash>(&mut self, item: &T) {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        self.state = self.state.wrapping_mul(hasher.finish());
    }
    pub fn remove<T: Hash>(&mut self, item: &T) {
        let mut hasher = DefaultHasher::new();
        item.hash(&mut hasher);
        let inv = hasher.finish();
        if inv != 0 {
            self.state = self.state.wrapping_div(inv);
        }
    }
    pub fn finalize(&self) -> u64 { self.state }
}

#[test]
fn test_add_and_finalize() {
    let mut muhash = MuHash::new();
    muhash.add(&123u64);
    let result = muhash.finalize();
    assert!(result > 1);
}

#[test]
fn test_add_remove() {
    let mut muhash = MuHash::new();
    muhash.add(&10u64);
    muhash.remove(&10u64);
    assert_eq!(muhash.finalize(), 1);
}
