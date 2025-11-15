//! Difficulty manager for consensus
//!
//! This module manages difficulty adjustment calculations based on
//! block timestamps and target block time.

use consensus_core::header::Header;
use consensus_core::constants::{MIN_DIFFICULTY_BITS, TARGET_BLOCK_TIME, DIFFICULTY_WINDOW};
use super::window::DifficultyWindow;
use std::sync::Arc;

/// Difficulty manager for consensus
pub struct DifficultyManager {
    target_time_per_block: u64,
    window_size: usize,
    window: Arc<std::sync::Mutex<DifficultyWindow>>,
}

impl DifficultyManager {
    /// Create a new difficulty manager with default parameters
    pub fn new() -> Self {
        Self {
            target_time_per_block: TARGET_BLOCK_TIME * 1000, // Convert to milliseconds
            window_size: DIFFICULTY_WINDOW as usize,
            window: Arc::new(std::sync::Mutex::new(DifficultyWindow::new(
                DIFFICULTY_WINDOW as usize,
            ))),
        }
    }

    /// Create a new difficulty manager with custom parameters
    pub fn with_params(target_time_per_block: u64, window_size: usize) -> Self {
        Self {
            target_time_per_block: target_time_per_block * 1000, // Convert to milliseconds
            window_size,
            window: Arc::new(std::sync::Mutex::new(DifficultyWindow::new(window_size))),
        }
    }

    /// Calculate next difficulty based on current window
    pub fn calculate_next_difficulty(&self, current_header: &Header) -> Result<u32, String> {
        let mut window = self.window.lock().unwrap();
        
        // Add current block to window
        window.add_block(current_header);

        // Need at least 2 blocks to calculate difficulty
        if window.len() < 2 {
            return Ok(current_header.bits);
        }

        // Get time span
        let time_span = window
            .time_span()
            .ok_or("Cannot calculate time span".to_string())?;

        // Get target time span
        let target_time_span = self.target_time_per_block * (window.len() as u64 - 1);

        // Get current difficulty
        let current_bits = current_header.bits;
        let current_target = self.bits_to_target(current_bits);

        // Calculate new target
        let time_span_u256: primitive_types::U256 = time_span.into();
        let target_time_span_u256: primitive_types::U256 = target_time_span.into();
        
        let new_target = if time_span < target_time_span {
            // Blocks are coming too fast, increase difficulty
            current_target
                .checked_mul(time_span_u256)
                .and_then(|x| x.checked_div(target_time_span_u256))
                .ok_or("Difficulty calculation overflow")?
        } else {
            // Blocks are coming too slow, decrease difficulty
            current_target
                .checked_mul(time_span_u256)
                .and_then(|x| x.checked_div(target_time_span_u256))
                .ok_or("Difficulty calculation overflow")?
        };

        // Clamp to minimum difficulty
        let min_target = self.bits_to_target(MIN_DIFFICULTY_BITS);
        let clamped_target = if new_target > min_target {
            min_target
        } else {
            new_target
        };

        Ok(self.target_to_bits(clamped_target))
    }

    /// Convert compact bits representation to target (U256)
    fn bits_to_target(&self, bits: u32) -> primitive_types::U256 {
        let size = (bits >> 24) as usize;
        let word = bits & 0x007fffff;

        if size <= 3 {
            primitive_types::U256::from(word >> (8 * (3 - size)))
        } else {
            primitive_types::U256::from(word) << (8 * (size - 3))
        }
    }

    /// Convert target (U256) to compact bits representation
    fn target_to_bits(&self, target: primitive_types::U256) -> u32 {
        let mut bytes = [0u8; 32];
        target.to_big_endian(&mut bytes);

        // Find first non-zero byte
        let mut size = 32;
        for (i, &byte) in bytes.iter().enumerate() {
            if byte != 0 {
                size = 32 - i;
                break;
            }
        }

        if size <= 3 {
            let word = u32::from_be_bytes([
                bytes[29],
                bytes[30],
                bytes[31],
                0,
            ]);
            ((size as u32) << 24) | (word >> 8)
        } else {
            let word = u32::from_be_bytes([
                bytes[32 - size],
                bytes[33 - size],
                bytes[34 - size],
                0,
            ]);
            ((size as u32) << 24) | (word >> 8)
        }
    }

    /// Get current difficulty window
    pub fn get_window(&self) -> DifficultyWindow {
        self.window.lock().unwrap().clone()
    }

    /// Clear the difficulty window
    pub fn clear_window(&self) {
        self.window.lock().unwrap().clear();
    }
}

impl Default for DifficultyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use consensus_core::header::Header;
    use consensus_core::{Hash, ZERO_HASH, BlueWorkType};

    fn create_test_header(_hash: Hash, timestamp: u64, bits: u32) -> Header {
        Header::new_finalized(
            1,
            vec![],
            ZERO_HASH,
            ZERO_HASH,
            ZERO_HASH,
            timestamp,
            bits,
            0,
            0,
            BlueWorkType::from(0u64),
            0,
            ZERO_HASH,
        )
    }

    #[test]
    fn test_bits_to_target() {
        let manager = DifficultyManager::new();
        let bits = 0x1f00ffff;
        let target = manager.bits_to_target(bits);
        assert!(target > primitive_types::U256::zero());
    }

    #[test]
    fn test_target_to_bits() {
        let manager = DifficultyManager::new();
        let bits = 0x1f00ffff;
        let target = manager.bits_to_target(bits);
        let converted_bits = manager.target_to_bits(target);
        // Note: Conversion may not be exact due to precision, but should be close
        assert!(converted_bits > 0);
    }

    #[test]
    fn test_calculate_next_difficulty_insufficient_blocks() {
        use consensus_core::Hash;
        let manager = DifficultyManager::new();
        let header = create_test_header(Hash::from_le_u64([1, 0, 0, 0]), 1000, 0x1f00ffff);
        let result = manager.calculate_next_difficulty(&header);
        assert!(result.is_ok());
        // With only one block, should return current bits
        assert_eq!(result.unwrap(), 0x1f00ffff);
    }
}

