use parking_lot::RwLock;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

/// Simple LRU-ish cache for small workloads (not production-grade)
pub struct LruCache<K, V> {
    capacity: usize,
    cache: RwLock<HashMap<K, CacheEntry<V>>>,
}

struct CacheEntry<V> {
    value: V,
    last_access: u64,
}

impl<K: Hash + Eq + Clone, V: Clone> LruCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write();
        if let Some(entry) = cache.get_mut(key) {
            entry.last_access = Self::now();
            return Some(entry.value.clone());
        }
        None
    }

    pub fn insert(&self, key: K, value: V) {
        let mut cache = self.cache.write();
        if cache.len() >= self.capacity && !cache.contains_key(&key) {
            // simple eviction: remove a random key (not ideal but simple)
            if let Some(k) = cache.keys().next().cloned() {
                cache.remove(&k);
            }
        }
        cache.insert(key, CacheEntry { value, last_access: Self::now() });
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.cache.write().remove(key).map(|e| e.value)
    }

    pub fn len(&self) -> usize {
        self.cache.read().len()
    }

    fn now() -> u64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
    }
}

/// Write-through cache wrapper
pub struct WriteThroughCache<K, V> {
    inner: Arc<LruCache<K, V>>,
}

impl<K: Hash + Eq + Clone, V: Clone> WriteThroughCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self { inner: Arc::new(LruCache::new(capacity)) }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key)
    }

    pub fn insert(&self, key: K, value: V) {
        self.inner.insert(key, value);
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        self.inner.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_cache() {
        let c = LruCache::new(2);
        c.insert(1u32, "one");
        c.insert(2u32, "two");
        assert_eq!(c.len(), 2);
        assert_eq!(c.get(&1u32), Some("one"));
        c.insert(3u32, "three");
        assert_eq!(c.len(), 2);
    }
}
