// public for benchmarks
#[doc(hidden)]
pub mod matrix;
#[cfg(feature = "wasm32-sdk")]
pub mod wasm;
#[doc(hidden)]
pub mod xoshiro;

use std::cmp::max;

use crate::matrix::Matrix;
use consensus_core::{constants, hashing, header::Header, BlockLevel};
use crypto_hashes::{Hash, HashWriter, PowB3Hash, PowFishHash};
use primitive_types::U256;

/// State is an intermediate data structure with pre-computed values to speed up mining.
pub struct State {
    pub(crate) matrix: Matrix,
    pub(crate) target: U256,
    // PRE_POW_HASH || TIME || 32 zero byte padding; without NONCE
    pub(crate) hasher: PowB3Hash,
    pub(crate) header_version: u16,
}

impl State {
    #[inline]
    pub fn new(header: &Header) -> Self {
        // Convert compact bits to full target U256
        let target = {
            let size = (header.bits >> 24) as usize;
            let word = header.bits & 0x007fffff;
            if size <= 3 {
                U256::from(word >> (8 * (3 - size)))
            } else {
                U256::from(word) << (8 * (size - 3))
            }
        };

        // Zero out the time and nonce to produce pre-pow hash.
        let pre_pow_hash = hashing::header::hash_override_nonce_time(header, 0, 0);
        // PRE_POW_HASH || TIME || 32 zero byte padding || NONCE
        //let hasher = PowHash::new(pre_pow_hash, header.timestamp);
        let hasher = PowB3Hash::new(pre_pow_hash, header.timestamp);
        let matrix = Matrix::generate(pre_pow_hash);
        //let fishhasher = PowFishHash::new();
        let header_version = header.version;

        Self { matrix, target, hasher, /*fishhasher,*/ header_version }
    }

    #[inline]
    fn calculate_pow_khashv1(&self, nonce: u64) -> U256 {
        // Hasher already contains PRE_POW_HASH || TIME || 32 zero byte padding; so only the NONCE is missing
        let hash = self.hasher.clone().finalize_with_nonce(nonce);
        let hash = self.matrix.heavy_hash(hash);
        // Convert resulting 32-byte hash into U256 (big-endian)
        U256::from_big_endian(hash.as_bytes())
    }

    #[inline]
    fn calculate_pow_khashv2plus(&self, nonce: u64) -> U256 {
        // TODO: implement v2 matrix+fish hashing. For now fallback to v1 behavior.
        let v1 = self.calculate_pow_khashv1(nonce);
        v1
    }

    #[inline]
    #[must_use]
    /// PRE_POW_HASH || TIME || 32 zero byte padding || NONCE
    pub fn calculate_pow(&self, nonce: u64) -> U256 {
        match self.header_version {
            constants::BLOCK_VERSION_KHASHV1 => self.calculate_pow_khashv1(nonce),
            constants::BLOCK_VERSION_KHASHV2 => self.calculate_pow_khashv2plus(nonce),
            _ => {
                // Fallback to v1
                self.calculate_pow_khashv1(nonce)
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn check_pow(&self, nonce: u64) -> (bool, U256) {
        let pow = self.calculate_pow(nonce);
        // The pow hash must be less or equal than the claimed target.
        (pow <= self.target, pow)
    }
}

pub fn calc_block_level(header: &Header, max_block_level: BlockLevel) -> BlockLevel {
    let (block_level, _) = calc_block_level_check_pow(header, max_block_level);
    block_level
}

pub fn calc_block_level_check_pow(header: &Header, max_block_level: BlockLevel) -> (BlockLevel, bool) {
    if header.parents_by_level.is_empty() {
        return (max_block_level, true); // Genesis has the max block level
    }

    let state = State::new(header);
    let (passed, pow) = state.check_pow(header.nonce);
    let block_level = calc_level_from_pow(pow, max_block_level);
    (block_level, passed)
}

pub fn calc_level_from_pow(pow: U256, max_block_level: BlockLevel) -> BlockLevel {
    // Compute an approximate "bits" value (index of most significant bit). If pow == 0 -> 0.
    let pow_bits: i64 = if pow.is_zero() {
        0
    } else {
        let mut be = [0u8; 32];
        pow.to_big_endian(&mut be);
        // find first non-zero byte
        let mut idx = 0usize;
        while idx < 32 && be[idx] == 0 {
            idx += 1;
        }
        let byte = be[idx];
        let leading = byte.leading_zeros() as usize;
        let bit_pos = ((32 - idx) * 8) - leading; // 1..=256
        bit_pos as i64
    };

    let signed_block_level = max_block_level as i64 - pow_bits;
    max(signed_block_level, 0) as BlockLevel
}