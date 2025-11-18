//! Cache layer using Redis

use redis::Client;
use std::sync::Arc;
use crate::error::Result;

pub struct Cache {
    client: Arc<Client>,
}

impl Cache {
    pub fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client: Arc::new(client),
        })
    }
    
    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let mut conn = self.client.get_async_connection().await?;
        let value: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        
        match value {
            Some(v) => {
                let decoded: T = serde_json::from_str(&v)?;
                Ok(Some(decoded))
            }
            None => Ok(None),
        }
    }
    
    pub async fn set<T: serde::Serialize>(&self, key: &str, value: &T, ttl: u64) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let serialized = serde_json::to_string(value)?;
        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl)
            .arg(serialized)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
    
    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        redis::cmd("DEL")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}

