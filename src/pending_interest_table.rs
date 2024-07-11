// Filename: pending_interest_table.rs

// =================================================
// Overview
// =================================================
// This file defines the PendingInterestTable struct, which is used to manage pending interests
// in an Information-Centric Network (ICN) node. The Pending Interest Table (PIT) keeps track
// of interest packets that have been forwarded but for which no data has yet been received.
// It stores information about the interfaces through which the interests were received and
// the time when the interests were added to the table. This helps in handling interest 
// timeouts and forwarding data packets to the correct interfaces when the data arrives.

// =================================================
// Imports
// =================================================

use std::collections::HashMap;             // HashMap is used to store pending interests.
use std::time::{Duration, Instant};        // Duration and Instant are used to manage time-related operations.

// =================================================
// Constants
// =================================================

const DEFAULT_INTEREST_LIFETIME: Duration = Duration::from_secs(4); // Default lifetime for an interest entry in the PIT.

// =================================================
// PitEntry Struct: Represents an Entry in the Pending Interest Table
// =================================================

// Each PitEntry contains a list of interfaces and a timestamp.
// The interfaces list stores the names of the interfaces through which the interest was received.
// The timestamp records when the interest was added to the table.
struct PitEntry {
    interfaces: Vec<String>,  // List of interfaces through which interests were received.
    timestamp: Instant,       // Timestamp of when the interest was added.
}

// =================================================
// PendingInterestTable Struct: Manages Pending Interests
// =================================================

// The PendingInterestTable struct contains a HashMap that stores pending interests.
// The keys in the HashMap are the names of the interests, and the values are PitEntry structs.
pub struct PendingInterestTable {
    entries: HashMap<String, PitEntry>, // Collection of pending interests.
}

// Implementation of the PendingInterestTable struct.
impl PendingInterestTable {
    // Create a new, empty Pending Interest Table.
    // This function initializes an empty HashMap to store the entries.
    pub fn new() -> Self {
        PendingInterestTable {
            entries: HashMap::new(),
        }
    }

    // Add an interest to the table, updating the timestamp and interfaces if it already exists.
    // If the interest is already in the table, update its timestamp and add the interface if it's new.
    // If the interest is not in the table, create a new entry with the current timestamp and the given interface.
    pub fn add_interest(&mut self, name: String, interface: &str) {
        self.entries
            .entry(name) // Try to find the interest in the HashMap.
            .and_modify(|e| { // If the interest is found, modify the existing entry.
                if !e.interfaces.contains(&interface.to_string()) { // Check if the interface is already in the list.
                    e.interfaces.push(interface.to_string()); // Add the interface to the list if it's new.
                }
                e.timestamp = Instant::now(); // Update the timestamp to the current time.
            })
            .or_insert(PitEntry { // If the interest is not found, create a new entry.
                interfaces: vec![interface.to_string()], // Initialize the interfaces list with the given interface.
                timestamp: Instant::now(), // Set the timestamp to the current time.
            });
    }

    // Remove an interest from the table.
    // This function deletes the entry corresponding to the given interest name.
    pub fn remove_interest(&mut self, name: &str) {
        self.entries.remove(name); // Remove the entry from the HashMap.
    }

    // Check if there is a pending interest for a given name.
    // This function returns true if the interest is in the table, and false otherwise.
    pub fn has_pending_interest(&self, name: &str) -> bool {
        self.entries.contains_key(name) // Check if the interest is in the HashMap.
    }

    // Get the list of incoming interfaces for a given interest name.
    // This function returns a clone of the interfaces list if the interest is found, or None otherwise.
    pub fn get_incoming_interfaces(&self, name: &str) -> Option<Vec<String>> {
        self.entries.get(name).map(|entry| entry.interfaces.clone()) // Get the interfaces list if the interest is found.
    }

    // Add an incoming interface to an existing interest entry.
    // This function adds the given interface to the list of interfaces for the specified interest, if the interest exists.
    pub fn add_incoming_interface(&mut self, name: &str, interface: &str) {
        if let Some(entry) = self.entries.get_mut(name) { // Check if the interest is in the table.
            if !entry.interfaces.contains(&interface.to_string()) { // Check if the interface is already in the list.
                entry.interfaces.push(interface.to_string()); // Add the interface to the list if it's new.
            }
        }
    }

    // Remove expired interests from the table.
    // This function deletes entries that have been in the table longer than the default interest lifetime.
    pub fn clear_expired(&mut self) {
        self.entries.retain(|_, entry| entry.timestamp.elapsed() < DEFAULT_INTEREST_LIFETIME); // Remove entries older than the default lifetime.
    }
}

// =================================================
// Unit Tests for PendingInterestTable
// =================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test the functionality of the PendingInterestTable.
    #[test]
    fn test_pending_interest_table() {
        let mut pit = PendingInterestTable::new(); // Create a new, empty Pending Interest Table.
        
        pit.add_interest("test".to_string(), "interface1"); // Add an interest to the table.
        assert!(pit.has_pending_interest("test")); // Check that the interest is in the table.
        
        pit.add_incoming_interface("test", "interface2"); // Add another interface to the interest.
        let interfaces = pit.get_incoming_interfaces("test").unwrap(); // Get the list of interfaces for the interest.
        assert_eq!(interfaces.len(), 2); // Check that the list contains two interfaces.
        assert!(interfaces.contains(&"interface1".to_string())); // Check that the first interface is in the list.
        assert!(interfaces.contains(&"interface2".to_string())); // Check that the second interface is in the list.
        
        pit.remove_interest("test"); // Remove the interest from the table.
        assert!(!pit.has_pending_interest("test")); // Check that the interest is no longer in the table.

        // Test clearing expired entries.
        pit.add_interest("test_expired".to_string(), "interface1"); // Add an interest to the table.
        std::thread::sleep(Duration::from_secs(5)); // Wait for the interest to expire.
        pit.clear_expired(); // Remove expired entries from the table.
        assert!(!pit.has_pending_interest("test_expired")); // Check that the expired interest is no longer in the table.
    }
}
