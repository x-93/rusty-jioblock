//! Address indexing and balance tracking

use std::sync::Arc;
use consensus_core::tx::Transaction;
use crate::database::Database;
use crate::error::Result;
use sqlx::Row;

pub struct AddressIndexer {
    pool: Arc<sqlx::SqlitePool>,
}

impl AddressIndexer {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            pool: Arc::new(database.pool().clone()),
        }
    }
    
    pub async fn update_from_transaction(&self, tx: &Transaction) -> Result<()> {
        let tx_hash = tx.hash().to_string();
        let timestamp = chrono::Utc::now().timestamp() as i64;
        
        // Process outputs (addresses receiving)
        for output in &tx.outputs {
            if let Some(address) = self.extract_address(&output.script_public_key) {
                self.update_address_received(&address, output.value, timestamp).await?;
            }
        }
        
        // Process inputs (addresses sending)
        for input in &tx.inputs {
            // Get the previous output to find the address
            if let Some((address, value)) = self.get_previous_output_address(&input.previous_outpoint.transaction_id.to_string(), input.previous_outpoint.index).await? {
                self.update_address_sent(&address, value, timestamp).await?;
            }
        }
        
        Ok(())
    }
    
    async fn update_address_received(&self, address: &str, value: u64, timestamp: i64) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO addresses (
                address, balance, total_received, received_count,
                tx_count, utxo_count, first_seen_timestamp, last_seen_timestamp
            ) VALUES ($1, $2, $3, 1, 1, 1, $4, $4)
            ON CONFLICT (address) DO UPDATE SET
                balance = addresses.balance + $2,
                total_received = addresses.total_received + $3,
                received_count = addresses.received_count + 1,
                tx_count = addresses.tx_count + 1,
                utxo_count = addresses.utxo_count + 1,
                last_seen_timestamp = GREATEST(addresses.last_seen_timestamp, $4),
                updated_at = NOW()
            "#,
        )
        .bind(address)
        .bind(value as i64)
        .bind(value as i64)
        .bind(timestamp)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
    
    async fn update_address_sent(&self, address: &str, value: u64, timestamp: i64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE addresses
            SET
                balance = balance - $1,
                total_sent = total_sent + $2,
                sent_count = sent_count + 1,
                tx_count = tx_count + 1,
                utxo_count = GREATEST(0, utxo_count - 1),
                last_seen_timestamp = GREATEST(last_seen_timestamp, $3),
                updated_at = NOW()
            WHERE address = $4
            "#,
        )
        .bind(value as i64)
        .bind(value as i64)
        .bind(timestamp)
        .bind(address)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
    
    async fn get_previous_output_address(&self, tx_hash: &str, index: u32) -> Result<Option<(String, u64)>> {
        let result = sqlx::query(
            r#"
            SELECT address, value
            FROM transaction_outputs
            WHERE tx_hash = $1 AND index = $2
            "#,
        )
        .bind(tx_hash)
        .bind(index as i32)
        .fetch_optional(&*self.pool)
        .await?;

        Ok(result.and_then(|r| {
            let address: Option<String> = r.try_get("address").ok()?;
            let value: i64 = r.try_get("value").ok()?;
            address.map(|addr| (addr, value as u64))
        }))
    }
    
    fn extract_address(&self, script_pub_key: &consensus_core::tx::ScriptPublicKey) -> Option<String> {
        if script_pub_key.script().is_empty() {
            None
        } else {
            Some(hex::encode(script_pub_key.script()))
        }
    }
}

