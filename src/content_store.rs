use std::collections::HashMap;
use std::time::{Duration, Instant};

const MAX_CACHE_SIZE: usize = 1000;
const DEFAULT_TTL: Duration = Duration::from_secs(3600); // 1 hour

struct CacheEntry {
    content: Vec<u8>,
    timestamp: Instant,
    ttl: Duration,
}

pub struct ContentStore {
    cache: HashMap<String, CacheEntry>,
}

impl ContentStore {
    pub fn new() -> Self {
        ContentStore {
            cache: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, content: Vec<u8>) {
        self.cache.insert(name, CacheEntry {
            content,
            timestamp: Instant::now(),
            ttl: DEFAULT_TTL,
        });

        // Evict oldest entries if cache size exceeds MAX_CACHE_SIZE
        if self.cache.len() > MAX_CACHE_SIZE {
            let oldest_key = self.cache
                .iter()
                .min_by_key(|(_, entry)| entry.timestamp)
                .map(|(k, _)| k.clone());

            if let Some(key) = oldest_key {
                self.cache.remove(&key);
            }
        }
    }

    pub fn get(&mut self, name: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.cache.get(name) {
            if entry.timestamp.elapsed() < entry.ttl {
                Some(entry.content.clone())
            } else {
                self.cache.remove(name);
                None
            }
        } else {
            None
        }
    }

    pub fn set_ttl(&mut self, name: &str, ttl: Duration) {
        if let Some(entry) = self.cache.get_mut(name) {
            entry.ttl = ttl;
        }
    }

    pub fn clear_expired(&mut self) {
        self.cache.retain(|_, entry| entry.timestamp.elapsed() < entry.ttl);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_store() {
        let mut cs = ContentStore::new();
        let content = vec![1, 2, 3, 4];
        cs.add("test".to_string(), content.clone());

        assert_eq!(cs.get("test"), Some(content));
        assert_eq!(cs.get("nonexistent"), None);

        // Test TTL
        cs.set_ttl("test", Duration::from_secs(1));
        std::thread::sleep(Duration::from_secs(2));
        assert_eq!(cs.get("test"), None);
    }
}