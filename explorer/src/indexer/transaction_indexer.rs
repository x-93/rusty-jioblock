//! Transaction indexing logic

use std::sync::Arc;
use consensus_core::{block::Block, tx::Transaction};
use crate::database::Database;
use crate::error::Result;

pub struct TransactionIndexer {
    pool: Arc<sqlx::SqlitePool>,
}

impl TransactionIndexer {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            pool: Arc::new(database.pool().clone()),
        }
    }
    
    pub async fn index(&self, tx: &Transaction, block: Option<&Block>) -> Result<()> {
        let hash = tx.hash().to_string();
        let block_hash = block.map(|b| b.header.hash.to_string());
        let block_height = block.and_then(|b| self.get_block_height(&b.header.hash.to_string()));
        let is_coinbase = tx.is_coinbase();

        // Calculate transaction value
        let value: u64 = tx.outputs.iter().map(|out| out.value).sum();
        let fee = if is_coinbase {
            None
        } else {
            // Calculate fee (inputs - outputs)
            let input_value: u64 = 0; // TODO: Calculate from previous outputs
            Some(input_value.saturating_sub(value) as i64)
        };

        // Insert transaction
        sqlx::query(
            r#"
            INSERT INTO transactions (
                hash, block_hash, block_height, index_in_block,
                version, lock_time, input_count, output_count,
                size, fee, value, timestamp, is_coinbase, is_confirmed
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            ON CONFLICT (hash) DO UPDATE SET
                block_hash = EXCLUDED.block_hash,
                block_height = EXCLUDED.block_height,
                is_confirmed = EXCLUDED.is_confirmed
            "#,
        )
        .bind(&hash)
        .bind(&block_hash)
        .bind(&block_height)
        .bind(&block.and_then(|b| b.transactions.iter().position(|t| t.hash() == tx.hash())).map(|i| i as i32))
        .bind(tx.version as i32)
        .bind(tx.lock_time as i64)
        .bind(tx.inputs.len() as i32)
        .bind(tx.outputs.len() as i32)
        .bind(self.calculate_tx_size(tx) as i32)
        .bind(&fee)
        .bind(value as i64)
        .bind(chrono::Utc::now().timestamp() as i64)
        .bind(is_coinbase)
        .bind(block.is_some())
        .execute(&*self.pool)
        .await?;

        // Index inputs
        for (idx, input) in tx.inputs.iter().enumerate() {
            self.index_input(&hash, idx, input).await?;
        }

        // Index outputs
        for (idx, output) in tx.outputs.iter().enumerate() {
            self.index_output(&hash, idx, output).await?;
        }

        Ok(())
    }
    
    async fn index_input(&self, tx_hash: &str, index: usize, input: &consensus_core::tx::TransactionInput) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO transaction_inputs (
                tx_hash, index, previous_outpoint_hash, previous_outpoint_index, sequence
            ) VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (tx_hash, index) DO NOTHING
            "#,
        )
        .bind(tx_hash)
        .bind(index as i32)
        .bind(input.previous_outpoint.transaction_id.to_string())
        .bind(input.previous_outpoint.index as i32)
        .bind(input.sequence as i64)
        .execute(&*self.pool)
        .await?;

        // Mark previous output as spent
        sqlx::query(
            r#"
            UPDATE transaction_outputs
            SET is_spent = TRUE, spent_by_tx_hash = $1, spent_by_input_index = $2
            WHERE tx_hash = $3 AND index = $4
            "#,
        )
        .bind(tx_hash)
        .bind(index as i32)
        .bind(input.previous_outpoint.transaction_id.to_string())
        .bind(input.previous_outpoint.index as i32)
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
    
    async fn index_output(&self, tx_hash: &str, index: usize, output: &consensus_core::tx::TransactionOutput) -> Result<()> {
        // Extract address from script public key (simplified)
        let address = self.extract_address(&output.script_public_key);

        sqlx::query(
            r#"
            INSERT INTO transaction_outputs (
                tx_hash, index, value, script_public_key_version,
                script_public_key_script, address
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (tx_hash, index) DO NOTHING
            "#,
        )
        .bind(tx_hash)
        .bind(index as i32)
        .bind(output.value as i64)
        .bind(output.script_public_key.version as i32)
        .bind(output.script_public_key.script())
        .bind(&address)
        .execute(&*self.pool)
        .await?;

        // Update address transaction mapping
        if let Some(ref addr) = address {
            sqlx::query(
                r#"
                INSERT INTO address_transactions (address, tx_hash, is_input, value)
                VALUES ($1, $2, FALSE, $3)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(addr)
            .bind(tx_hash)
            .bind(output.value as i64)
            .execute(&*self.pool)
            .await?;
        }

        Ok(())
    }
    
    fn extract_address(&self, script_pub_key: &consensus_core::tx::ScriptPublicKey) -> Option<String> {
        // Simplified address extraction
        // In production, this should properly decode the script
        if script_pub_key.script().is_empty() {
            None
        } else {
            Some(hex::encode(&script_pub_key.script()))
        }
    }
    
    fn calculate_tx_size(&self, tx: &Transaction) -> usize {
        std::mem::size_of_val(tx)
    }
    
    fn get_block_height(&self, hash: &str) -> Option<i64> {
        // TODO: Implement synchronous block height lookup
        // For now, return None - this will be fixed when we have a proper cache
        None
    }
}

