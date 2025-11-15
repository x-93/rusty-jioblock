use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConsensusError {
    #[error("Invalid block version")]
    InvalidBlockVersion,

    #[error("Block exceeds maximum mass")]
    ExceedsMaxBlockMass,

    #[error("Invalid merkle root")]
    InvalidMerkleRoot,

    #[error("Invalid proof of work")]
    InvalidProofOfWork,

    #[error("Invalid coinbase transaction")]
    InvalidCoinbaseTransaction,

    #[error("Empty transaction list")]
    EmptyTransactionList,

    #[error("Invalid transaction")]
    InvalidTransaction,

    #[error("Invalid script")]
    InvalidScript,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Double spend attempt")]
    DoubleSpend,

    #[error("Invalid UTXO reference")]
    InvalidUtxoReference,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Invalid block parent")]
    InvalidBlockParent,

    #[error("Invalid DAG structure")]
    InvalidDagStructure,

    #[error("Invalid difficulty target")]
    InvalidDifficultyTarget,

    #[error("Invalid timestamp")]
    InvalidTimestamp,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Other error: {0}")]
    Other(String),
}