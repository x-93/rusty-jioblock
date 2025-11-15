//! Difficulty window management
//!
//! This module manages a sliding window of blocks for difficulty adjustment
//! calculations.

use consensus_core::header::Header;
use consensus_core::Hash;
use std::collections::VecDeque;

/// Difficulty window for adjustment calculations
#[derive(Clone)]
pub struct DifficultyWindow {
    window_size: usize,
    blocks: VecDeque<(Hash, u64, u32)>, // (hash, timestamp, bits)
}

impl DifficultyWindow {
    /// Create a new difficulty window
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            blocks: VecDeque::with_capacity(window_size),
        }
    }

    /// Add a block to the window
    pub fn add_block(&mut self, header: &Header) {
        if self.blocks.len() >= self.window_size {
            self.blocks.pop_front();
        }
        self.blocks.push_back((header.hash, header.timestamp, header.bits));
    }

    /// Get the window size
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Get the number of blocks in the window
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Check if the window is full
    pub fn is_full(&self) -> bool {
        self.blocks.len() >= self.window_size
    }

    /// Get timestamps from the window
    pub fn timestamps(&self) -> Vec<u64> {
        self.blocks.iter().map(|(_, timestamp, _)| *timestamp).collect()
    }

    /// Get bits from the window
    pub fn bits(&self) -> Vec<u32> {
        self.blocks.iter().map(|(_, _, bits)| *bits).collect()
    }

    /// Get the first timestamp in the window
    pub fn first_timestamp(&self) -> Option<u64> {
        self.blocks.front().map(|(_, timestamp, _)| *timestamp)
    }

    /// Get the last timestamp in the window
    pub fn last_timestamp(&self) -> Option<u64> {
        self.blocks.back().map(|(_, timestamp, _)| *timestamp)
    }

    /// Calculate time span of the window
    pub fn time_span(&self) -> Option<u64> {
        match (self.first_timestamp(), self.last_timestamp()) {
            (Some(first), Some(last)) => {
                if last > first {
                    Some(last - first)
                } else {
                    Some(1) // Minimum 1ms to avoid division by zero
                }
            }
            _ => None,
        }
    }

    /// Clear the window
    pub fn clear(&mut self) {
        self.blocks.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_window_add_block() {
        let mut window = DifficultyWindow::new(10);
        let header = create_test_header(Hash::from_le_u64([1, 0, 0, 0]), 1000, 0x1f00ffff);
        window.add_block(&header);
        assert_eq!(window.len(), 1);
    }

    #[test]
    fn test_window_size_limit() {
        let mut window = DifficultyWindow::new(3);
        for i in 0..5 {
            let header = create_test_header(
                Hash::from_le_u64([i as u64, 0, 0, 0]),
                1000 + i * 1000,
                0x1f00ffff,
            );
            window.add_block(&header);
        }
        assert_eq!(window.len(), 3);
    }

    #[test]
    fn test_time_span() {
        let mut window = DifficultyWindow::new(10);
        window.add_block(&create_test_header(Hash::from_le_u64([1, 0, 0, 0]), 1000, 0x1f00ffff));
        window.add_block(&create_test_header(Hash::from_le_u64([2, 0, 0, 0]), 2000, 0x1f00ffff));
        assert_eq!(window.time_span(), Some(1000));
    }
}

