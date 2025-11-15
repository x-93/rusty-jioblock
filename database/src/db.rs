use crate::errors::{DbError, DbResult};
use rocksdb::{DB, Options, ColumnFamilyDescriptor, IteratorMode, WriteBatch};
use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;

pub const CF_BLOCKS: &str = "blocks";
pub const CF_HEADERS: &str = "headers";
pub const CF_TRANSACTIONS: &str = "transactions";
pub const CF_UTXOS: &str = "utxos";
pub const CF_GHOSTDAG: &str = "ghostdag";
pub const CF_REACHABILITY: &str = "reachability";
pub const CF_METADATA: &str = "metadata";
pub const CF_BLOCK_RELATIONS: &str = "block_relations";

pub struct Database {
    db: Arc<DB>,
    is_closed: Arc<RwLock<bool>>,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> DbResult<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(10000);
        opts.set_keep_log_file_num(10);
        opts.set_max_background_jobs(4);
        opts.set_bytes_per_sync(1048576);
        opts.increase_parallelism(4);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        opts.set_write_buffer_size(64 * 1024 * 1024);
        opts.set_max_write_buffer_number(3);

        let cf_names = vec![
            CF_BLOCKS,
            CF_HEADERS,
            CF_TRANSACTIONS,
            CF_UTXOS,
            CF_GHOSTDAG,
            CF_REACHABILITY,
            CF_METADATA,
            CF_BLOCK_RELATIONS,
        ];

        let cf_descriptors: Vec<_> = cf_names
            .iter()
            .map(|name| ColumnFamilyDescriptor::new(*name, Options::default()))
            .collect();

        let db = DB::open_cf_descriptors(&opts, path, cf_descriptors)?;
        Ok(Self { db: Arc::new(db), is_closed: Arc::new(RwLock::new(false)) })
    }

    fn check_closed(&self) -> DbResult<()> {
        if *self.is_closed.read() { return Err(DbError::DatabaseClosed); }
        Ok(())
    }

    fn get_cf_handle(&self, cf_name: &str) -> DbResult<&rocksdb::ColumnFamily> {
        self.db.cf_handle(cf_name)
            .ok_or_else(|| DbError::ColumnFamilyNotFound(cf_name.to_string()))
    }

    pub fn put(&self, cf_name: &str, key: &[u8], value: &[u8]) -> DbResult<()> {
        self.check_closed()?;
        let cf = self.get_cf_handle(cf_name)?;
        self.db.put_cf(cf, key, value)?;
        Ok(())
    }

    pub fn get(&self, cf_name: &str, key: &[u8]) -> DbResult<Option<Vec<u8>>> {
        self.check_closed()?;
        let cf = self.get_cf_handle(cf_name)?;
        Ok(self.db.get_cf(cf, key)?)
    }

    pub fn delete(&self, cf_name: &str, key: &[u8]) -> DbResult<()> {
        self.check_closed()?;
        let cf = self.get_cf_handle(cf_name)?;
        self.db.delete_cf(cf, key)?;
        Ok(())
    }

    pub fn exists(&self, cf_name: &str, key: &[u8]) -> DbResult<bool> {
        self.check_closed()?;
        let cf = self.get_cf_handle(cf_name)?;
        Ok(self.db.get_pinned_cf(cf, key)?.is_some())
    }

    pub fn batch(&self) -> WriteBatch { WriteBatch::default() }

    pub fn write_batch(&self, batch: WriteBatch) -> DbResult<()> { self.check_closed()?; self.db.write(batch)?; Ok(()) }

    pub fn iterator(&self, cf_name: &str, mode: IteratorMode) -> DbResult<rocksdb::DBIteratorWithThreadMode<'_, DB>> {
        self.check_closed()?;
        let cf = self.get_cf_handle(cf_name)?;
        Ok(self.db.iterator_cf(cf, mode))
    }

    pub fn close(&self) { *self.is_closed.write() = true; }

    pub fn stats(&self) -> String { self.db.property_value("rocksdb.stats").unwrap_or_default().unwrap_or_default() }

    pub fn compact(&self, cf_name: &str) -> DbResult<()> {
        let cf = self.get_cf_handle(cf_name)?;
        self.db.compact_range_cf(cf, None::<&[u8]>, None::<&[u8]>);
        Ok(())
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self { db: self.db.clone(), is_closed: self.is_closed.clone() }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_open_put_get() {
        let tmp = TempDir::new().unwrap();
        let db = Database::open(tmp.path()).unwrap();
        db.put(CF_METADATA, b"k", b"v").unwrap();
        let v = db.get(CF_METADATA, b"k").unwrap();
        assert_eq!(v, Some(b"v".to_vec()));
    }
}
