use crate::KType;

/// Current block version
pub const BLOCK_VERSION: u16 = 1;

/// Block version using KHash v1 algorithm
pub const BLOCK_VERSION_KHASHV1: u16 = 1;

/// Block version using KHash v2+ algorithm
pub const BLOCK_VERSION_KHASHV2: u16 = 2;

/// Total supply in Jiocoins (21 billion)
pub const TOTAL_SUPPLY: u64 = 21_000_000_000;

/// Block subsidy halving interval (in blocks)
pub const SUBSIDY_HALVING_INTERVAL: u64 = 210_000;

/// Initial block reward in Jiocoins
pub const INITIAL_BLOCK_REWARD: u64 = 50;

/// Target block time in seconds
pub const TARGET_BLOCK_TIME: u64 = 60;

/// Maximum time difference allowed between blocks (2 hours)
pub const MAX_BLOCK_TIME_DIFFERENCE: u64 = 7200;

/// Difficulty adjustment window (blocks)
pub const DIFFICULTY_WINDOW: u64 = 144;

/// GhostDAG K parameter - maximum number of blocks in anticone for blue selection
pub const GHOSTDAG_K: KType = 18;

/// Minimum difficulty bits (maximum target)
pub const MIN_DIFFICULTY_BITS: u32 = 0x1f00_ffff;

/// Genesis block timestamp
pub const GENESIS_BLOCK_TIMESTAMP: u64 = 1699545600000; // November 9, 2023 UTC

/// Maximum block mass (in grams, limiting block size)
pub const MAX_BLOCK_MASS: u64 = 1_000_000;

/// Minimum transaction fee rate (jiocoins per gram)
pub const MIN_TRANSACTION_FEE_RATE: u64 = 1;

/// Coinbase maturity in DAA score units (number of daa-score units before a coinbase output is spendable)
/// Typical value is 100 (Bitcoin uses 100 blocks) but in DAA-based systems this can be expressed in daa score
pub const COINBASE_MATURITY: u64 = 100;

/// Number of sompi (base units) in one Jiocoin
pub const SOMPI_PER_JIO: u64 = 100_000_000;

/// Mass factor for transient data (per byte)
pub const TRANSIENT_BYTE_TO_MASS_FACTOR: u64 = 10;

/// Mass parameter for storage calculations
pub const STORAGE_MASS_PARAMETER: u64 = 100;