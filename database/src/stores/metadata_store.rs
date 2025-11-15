use crate::{Database, DbResult};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataEntry {
    pub key: String,
    pub value: Vec<u8>,
}

pub struct MetadataStore {
    db: Arc<Database>,
}

impl MetadataStore {
    pub fn new(db: Arc<Database>) -> Self { Self { db } }

    pub fn put(&self, key: &str, value: &[u8]) -> DbResult<()> {
        self.db.put(crate::db::CF_METADATA, key.as_bytes(), value)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> DbResult<Option<Vec<u8>>> {
        self.db.get(crate::db::CF_METADATA, key.as_bytes())
    }

    pub fn delete(&self, key: &str) -> DbResult<()> {
        self.db.delete(crate::db::CF_METADATA, key.as_bytes())?;
        Ok(())
    }
}
