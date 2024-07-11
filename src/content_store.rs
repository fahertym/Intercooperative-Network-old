// Filename: content_store.rs

// =================================================
// Overview
// =================================================
// This file defines the ContentStore struct, which is used to manage cached content
// in an Information-Centric Network (ICN) node. The Content Store (CS) is responsible
// for storing data packets that have been received, allowing them to be quickly
// retrieved in response to future interest packets. The CS uses a cache eviction
// strategy to manage the size of the cache and a Time-To-Live (TTL) mechanism to
// handle content expiration.

// =================================================
// Imports
// =================================================

use std::collections::HashMap;            // HashMap is used to store cached content.
use std::time::{Duration, Instant};       // Duration and Instant are used to manage time-related operations.

// =================================================
// Constants
// =================================================

const MAX_CACHE_SIZE: usize = 1000;               // Maximum number of entries in the content store.
const DEFAULT_TTL: Duration = Duration::from_secs(3600); // Default Time-To-Live (TTL) for cached content (1 hour).

// =================================================
// CacheEntry Struct: Represents an Entry in the Content Store
// =================================================

// Each CacheEntry contains the content data, a timestamp, and a TTL.
// The content field stores the actual data bytes.
// The timestamp records when the content was added to the store.
// The TTL determines how long the content should be retained.
struct CacheEntry {
    content: Vec<u8>,     // The actual content data.
    timestamp: Instant,   // Timestamp of when the content was added.
    ttl: Duration,        // Time-To-Live for the content.
}

// =================================================
// ContentStore Struct: Manages Cached Content
// =================================================

// The ContentStore struct contains a HashMap that stores cached content.
// The keys in the HashMap are the content names, and the values are CacheEntry structs.
pub struct ContentStore {
    cache: HashMap<String, CacheEntry>, // Collection of cached content.
}

// Implementation of the ContentStore struct.
impl ContentStore {
    // Create a new, empty Content Store.
    // This function initializes an empty HashMap to store the cached content.
    pub fn new() -> Self {
        ContentStore {
            cache: HashMap::new(),
        }
    }

    // Add content to the store, associating it with the given name.
    // If the cache size exceeds the maximum limit, the oldest entry is evicted.
    pub fn add(&mut self, name: String, content: Vec<u8>) {
        self.cache.insert(name, CacheEntry {
            content,
            timestamp: Instant::now(),
            ttl: DEFAULT_TTL,
        });

        if self.cache.len() > MAX_CACHE_SIZE { // Check if the cache size exceeds the maximum limit.
            self.evict_oldest(); // Evict the oldest entry if the cache is full.
        }
    }

    // Retrieve content from the store by name.
    // If the content is found and has not expired, it is returned.
    // If the content has expired, it is removed from the store and None is returned.
    pub fn get(&mut self, name: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.cache.get(name) {
            if entry.timestamp.elapsed() < entry.ttl { // Check if the content has not expired.
                Some(entry.content.clone()) // Return the content if it is still valid.
            } else {
                self.cache.remove(name); // Remove the expired content from the store.
                None // Return None if the content has expired.
            }
        } else {
            None // Return None if the content is not found in the store.
        }
    }

    // Set a custom TTL for a specific content entry.
    // This function updates the TTL for the given content name.
    pub fn set_ttl(&mut self, name: &str, ttl: Duration) {
        if let Some(entry) = self.cache.get_mut(name) {
            entry.ttl = ttl; // Update the TTL for the content entry.
        }
    }

    // Remove expired content entries from the store.
    // This function deletes entries that have exceeded their TTL.
    pub fn clear_expired(&mut self) {
        self.cache.retain(|_, entry| entry.timestamp.elapsed() < entry.ttl); // Remove entries older than their TTL.
    }

    // Evict the oldest content entry from the store.
    // This function removes the entry with the oldest timestamp.
    fn evict_oldest(&mut self) {
        if let Some(oldest) = self.cache.iter()
            .min_by_key(|(_, entry)| entry.timestamp) // Find the entry with the oldest timestamp.
            .map(|(k, _)| k.clone()) {
            self.cache.remove(&oldest); // Remove the oldest entry from the store.
        }
    }
}

// =================================================
// Unit Tests for ContentStore
// =================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test the functionality of the ContentStore.
    #[test]
    fn test_content_store() {
        let mut cs = ContentStore::new(); // Create a new, empty Content Store.
        let content = vec![1, 2, 3, 4]; // Define some test content.
        cs.add("test".to_string(), content.clone()); // Add the content to the store.

        assert_eq!(cs.get("test"), Some(content)); // Check that the content can be retrieved.
        assert_eq!(cs.get("nonexistent"), None); // Check that a non-existent content returns None.

        // Test setting a custom TTL.
        cs.set_ttl("test", Duration::from_secs(1)); // Set a short TTL for the test content.
        std::thread::sleep(Duration::from_secs(2)); // Wait for the content to expire.
        assert_eq!(cs.get("test"), None); // Check that the expired content is no longer retrievable.

        // Test clearing expired entries.
        cs.add("test2".to_string(), vec![5, 6, 7, 8]); // Add another content entry.
        cs.set_ttl("test2", Duration::from_secs(10)); // Set a longer TTL for the new content.
        cs.clear_expired(); // Remove expired entries from the store.
        assert_eq!(cs.get("test2"), Some(vec![5, 6, 7, 8])); // Check that the non-expired content is still retrievable.
    }
}
