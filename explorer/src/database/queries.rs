//! Database query functions

use std::sync::Arc;
use crate::models::*;
use crate::error::Result;

pub struct BlockQueries;

impl BlockQueries {
    pub async fn get_by_hash(pool: Arc<sqlx::SqlitePool>, hash: &str) -> Result<Option<BlockSummary>> {
        let block = sqlx::query_as::<_, BlockSummary>(
            r#"
            SELECT
                hash,
                height,
                timestamp,
                tx_count,
                size,
                coinbase_value,
                (SELECT COUNT(*) FROM block_parents WHERE block_hash = ?) as parent_count,
                blue_score
            FROM blocks
            WHERE hash = ?
            "#
        )
        .bind(hash)
        .bind(hash)
        .fetch_optional(&*pool)
        .await?;

        Ok(block)
    }
    
    pub async fn get_by_height(pool: Arc<sqlx::SqlitePool>, height: i64) -> Result<Option<BlockSummary>> {
        let block = sqlx::query_as::<_, BlockSummary>(
            r#"
            SELECT
                hash,
                height,
                timestamp,
                tx_count,
                size,
                coinbase_value,
                (SELECT COUNT(*) FROM block_parents WHERE block_hash = blocks.hash) as parent_count,
                blue_score
            FROM blocks
            WHERE height = ?
            "#
        )
        .bind(height)
        .fetch_optional(&*pool)
        .await?;

        Ok(block)
    }
    
    pub async fn list_recent(pool: Arc<sqlx::SqlitePool>, limit: i64, offset: i64) -> Result<Vec<BlockSummary>> {
        let blocks = sqlx::query_as::<_, BlockSummary>(
            r#"
            SELECT
                hash,
                height,
                timestamp,
                tx_count,
                size,
                coinbase_value,
                (SELECT COUNT(*) FROM block_parents WHERE block_hash = blocks.hash) as parent_count,
                blue_score
            FROM blocks
            ORDER BY height DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*pool)
        .await?;

        Ok(blocks)
    }

    pub async fn count(pool: Arc<sqlx::SqlitePool>) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) as count FROM blocks"
        )
        .fetch_one(&*pool)
        .await?;

        Ok(count)
    }
}

pub struct TransactionQueries;

impl TransactionQueries {
    pub async fn get_by_hash(pool: Arc<sqlx::SqlitePool>, hash: &str) -> Result<Option<TransactionSummary>> {
        let tx = sqlx::query_as::<_, TransactionSummary>(
            r#"
            SELECT
                hash,
                block_hash,
                block_height,
                timestamp,
                input_count,
                output_count,
                value,
                fee,
                size,
                is_coinbase,
                is_confirmed,
                confirmation_count
            FROM transactions
            WHERE hash = ?
            "#
        )
        .bind(hash)
        .fetch_optional(&*pool)
        .await?;

        Ok(tx)
    }
    
    pub async fn list_recent(pool: Arc<sqlx::SqlitePool>, limit: i64, offset: i64) -> Result<Vec<TransactionSummary>> {
        let txs = sqlx::query_as::<_, TransactionSummary>(
            r#"
            SELECT
                hash,
                block_hash,
                block_height,
                timestamp,
                input_count,
                output_count,
                value,
                fee,
                size,
                is_coinbase,
                is_confirmed,
                confirmation_count
            FROM transactions
            ORDER BY timestamp DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*pool)
        .await?;

        Ok(txs)
    }
    
    pub async fn list_pending(pool: Arc<sqlx::SqlitePool>) -> Result<Vec<TransactionSummary>> {
        let txs = sqlx::query_as::<_, TransactionSummary>(
            r#"
            SELECT
                hash,
                block_hash,
                block_height,
                timestamp,
                input_count,
                output_count,
                value,
                fee,
                size,
                is_coinbase,
                is_confirmed,
                confirmation_count
            FROM transactions
            WHERE is_confirmed = FALSE
            ORDER BY timestamp DESC
            "#
        )
        .fetch_all(&*pool)
        .await?;

        Ok(txs)
    }

    pub async fn count(pool: Arc<sqlx::SqlitePool>) -> Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) as count FROM transactions"
        )
        .fetch_one(&*pool)
        .await?;

        Ok(count)
    }
}

pub struct AddressQueries;

impl AddressQueries {
    pub async fn get_summary(pool: Arc<sqlx::SqlitePool>, address: &str) -> Result<Option<AddressSummary>> {
        let addr = sqlx::query_as::<_, AddressSummary>(
            r#"
            SELECT
                address,
                balance,
                tx_count,
                received_count,
                sent_count,
                total_received,
                total_sent,
                utxo_count,
                first_seen_timestamp as first_seen,
                last_seen_timestamp as last_seen
            FROM addresses
            WHERE address = ?
            "#
        )
        .bind(address)
        .fetch_optional(&*pool)
        .await?;

        Ok(addr)
    }
    
    pub async fn get_transactions(
        pool: Arc<sqlx::SqlitePool>,
        address: &str,
        limit: i64,
        offset: i64
    ) -> Result<Vec<TransactionSummary>> {
        let txs = sqlx::query_as::<_, TransactionSummary>(
            r#"
            SELECT DISTINCT
                t.hash,
                t.block_hash,
                t.block_height,
                t.timestamp,
                t.input_count,
                t.output_count,
                t.value,
                t.fee,
                t.size,
                t.is_coinbase,
                t.is_confirmed,
                t.confirmation_count
            FROM transactions t
            INNER JOIN address_transactions at ON t.hash = at.tx_hash
            WHERE at.address = ?
            ORDER BY t.timestamp DESC
            LIMIT ? OFFSET ?
            "#
        )
        .bind(address)
        .bind(limit)
        .bind(offset)
        .fetch_all(&*pool)
        .await?;

        Ok(txs)
    }
}

