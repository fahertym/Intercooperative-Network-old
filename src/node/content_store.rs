use std::time::{Duration, Instant};
use lru::LruCache;

const MAX_CACHE_SIZE: usize = 1000;
const DEFAULT_TTL: Duration = Duration::from_secs(3600);

pub struct CacheEntry {
    content: Vec<u8>,
    timestamp: Instant,
    ttl: Duration,
}

pub struct ContentStore {
    cache: LruCache<String, CacheEntry>,
}

impl ContentStore {
    pub fn new() -> Self {
        ContentStore {
            cache: LruCache::new(MAX_CACHE_SIZE),
        }
    }

    pub fn add(&mut self, name: String, content: Vec<u8>) {
        let entry = CacheEntry {
            content,
            timestamp: Instant::now(),
            ttl: DEFAULT_TTL,
        };
        self.cache.put(name, entry);
    }

    pub fn get_and_pop(&mut self, name: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.cache.get(name) {
            let content = entry.content.clone(); // Clone the content to return later
            self.cache.pop(name); // Now, this works because the immutable borrow is out of scope
            Some(content)
        } else {
            None
        }
    }

    pub fn remove_expired(&mut self) {
        let now = Instant::now();
        let expired_keys: Vec<String> = self.cache
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.timestamp) >= entry.ttl)
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.cache.pop(&key);
        }
    }

    pub fn set_ttl(&mut self, name: &str, ttl: Duration) {
        if let Some(entry) = self.cache.get_mut(name) {
            entry.ttl = ttl;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
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

        assert_eq!(cs.get_and_pop("test"), Some(content));
        assert_eq!(cs.get_and_pop("nonexistent"), None);

        cs.add("test2".to_string(), vec![5, 6, 7, 8]);
        assert!(!cs.is_empty());

        cs.remove_expired();
        assert_eq!(cs.get_and_pop("test2"), Some(vec![5, 6, 7, 8]));
    }
}
