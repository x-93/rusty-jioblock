//! Difficulty adjustment module for consensus
//!
//! This module provides difficulty calculation and adjustment based on
//! block timestamps and target block time.

pub mod manager;
pub mod window;

pub use manager::DifficultyManager;
pub use window::DifficultyWindow;

