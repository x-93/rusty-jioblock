use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("RocksDB error: {0}")]
    RocksDb(#[from] rocksdb::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Key not found: {0}")]
    NotFound(String),
    
    #[error("Invalid data: {0}")]
    InvalidData(String),
    
    #[error("Column family not found: {0}")]
    ColumnFamilyNotFound(String),
    
    #[error("Database is closed")]
    DatabaseClosed,
    
    #[error("Cache error: {0}")]
    CacheError(String),
}

pub type DbResult<T> = Result<T, DbError>;

impl From<bincode::Error> for DbError {
    fn from(err: bincode::Error) -> Self {
        DbError::Serialization(err.to_string())
    }
}
