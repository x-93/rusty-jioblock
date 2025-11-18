//! Block indexing logic

use std::sync::Arc;
use consensus_core::block::Block;
use crate::database::Database;
use crate::error::Result;

pub struct BlockIndexer {
    pool: Arc<sqlx::SqlitePool>,
}

impl BlockIndexer {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            pool: Arc::new(database.pool().clone()),
        }
    }
    
    pub async fn index(&self, block: &Block) -> Result<()> {
        let hash = block.header.hash.to_string();
        let height = block.header.daa_score as i64;
        
        // Insert block
        sqlx::query(
            r#"
            INSERT INTO blocks (
                hash, height, version, timestamp, bits, nonce,
                merkle_root, accepted_id_merkle_root, utxo_commitment,
                daa_score, blue_score, blue_work, pruning_point,
                size, tx_count, coinbase_value
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            ON CONFLICT (hash) DO UPDATE SET
                height = EXCLUDED.height,
                timestamp = EXCLUDED.timestamp,
                tx_count = EXCLUDED.tx_count
            "#,
        )
        .bind(&hash)
        .bind(height)
        .bind(block.header.version as i32)
        .bind(block.header.timestamp as i64)
        .bind(block.header.bits as i32)
        .bind(block.header.nonce as i64)
        .bind(block.header.hash_merkle_root.to_string())
        .bind(block.header.accepted_id_merkle_root.to_string())
        .bind(block.header.utxo_commitment.to_string())
        .bind(block.header.daa_score as i64)
        .bind(block.header.blue_score as i64)
        .bind(format!("{}", block.header.blue_work))
        .bind(block.header.pruning_point.to_string())
        .bind(self.calculate_block_size(block) as i32)
        .bind(block.transactions.len() as i32)
        .bind(self.get_coinbase_value(block) as i64)
        .execute(&*self.pool)
        .await?;
        
        // Index block parents
        self.index_parents(&hash, &block.header.parents_by_level).await?;
        
        Ok(())
    }
    
    async fn index_parents(&self, block_hash: &str, parents_by_level: &[Vec<consensus_core::Hash>]) -> Result<()> {
        for (level, parents) in parents_by_level.iter().enumerate() {
            for parent in parents {
                sqlx::query(
                    r#"
                    INSERT INTO block_parents (block_hash, parent_hash, level)
                    VALUES ($1, $2, $3)
                    ON CONFLICT DO NOTHING
                    "#,
                )
                .bind(block_hash)
                .bind(parent.to_string())
                .bind(level as i32)
                .execute(&*self.pool)
                .await?;
            }
        }
        Ok(())
    }
    

    
    fn calculate_block_size(&self, block: &Block) -> usize {
        // Approximate block size
        std::mem::size_of_val(block) + block.transactions.iter().map(|tx| std::mem::size_of_val(tx)).sum::<usize>()
    }
    
    fn get_coinbase_value(&self, block: &Block) -> u64 {
        if let Some(coinbase) = block.transactions.first() {
            coinbase.outputs.iter().map(|out| out.value).sum()
        } else {
            0
        }
    }
}

