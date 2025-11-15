use crate::{Database, DbResult};
use consensus_core::Hash;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachabilityData {
    pub interval_start: u64,
    pub interval_end: u64,
    pub height: u64,
}

pub struct ReachabilityStore {
    db: Arc<Database>,
}

impl ReachabilityStore {
    pub fn new(db: Arc<Database>) -> Self { Self { db } }

    pub fn put_interval(&self, hash: &Hash, data: &ReachabilityData) -> DbResult<()> {
        let serialized = bincode::serialize(data)?;
        self.db.put(crate::db::CF_REACHABILITY, hash.as_bytes(), &serialized)?;
        Ok(())
    }

    pub fn get_interval(&self, hash: &Hash) -> DbResult<Option<ReachabilityData>> {
        if let Some(bytes) = self.db.get(crate::db::CF_REACHABILITY, hash.as_bytes())? {
            Ok(Some(bincode::deserialize(&bytes)?))
        } else { Ok(None) }
    }
}
