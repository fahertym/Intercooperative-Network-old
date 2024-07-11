// Filename: src/consensus/consensus.rs

// ===============================================
// Consensus Mechanism Implementation
// ===============================================
// This file contains the implementation of the consensus mechanism
// used in the blockchain. It includes the structures and functions
// necessary to achieve agreement on the state of the blockchain.
//
// Key concepts:
// - Consensus: The process of achieving agreement on the state of the blockchain
// - Proof of Contribution (PoC): A consensus mechanism that rewards nodes
//   based on their contributions to the network

use chrono::{DateTime, Utc};
// use rand::Rng;
use rand::distributions::{Distribution, Uniform};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// ===============================================
// Constants
// ===============================================

const DECAY_RATE: f64 = 0.1; // The rate at which reputation decays over time

// ===============================================
// Structures and Enums
// ===============================================

// Represents a member in the consensus mechanism
#[derive(Clone, Serialize, Deserialize)]
pub struct Member {
    pub id: String,           // Unique identifier for the member
    pub reputation: f64,      // Reputation score of the member
    pub last_decay: DateTime<Utc>, // Timestamp of the last reputation decay
}

// Represents the consensus mechanism
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Consensus {
    pub members: HashMap<String, Member>, // List of members in the consensus mechanism
}

impl Consensus {
    // Create a new Consensus instance
    pub fn new() -> Self {
        Consensus {
            members: HashMap::new(),
        }
    }

    // Add a new member to the consensus mechanism
    pub fn add_member(&mut self, id: String) {
        self.members.insert(
            id.clone(),
            Member {
                id,
                reputation: 1.0, // Default reputation
                last_decay: Utc::now(),
            },
        );
    }

    // Update the reputation of a member
    pub fn update_reputation(&mut self, id: &str, change: f64) {
        if let Some(member) = self.members.get_mut(id) {
            member.reputation += change;
        }
    }

    // Get the reputation of a member
    pub fn get_reputation(&self, id: &str) -> Option<f64> {
        self.members.get(id).map(|member| member.reputation)
    }

    // Select a proposer for the next block based on reputation
    pub fn select_proposer(&self) -> Option<String> {
        let total_reputation: f64 = self.members.values().map(|member| member.reputation).sum();
        let mut rng = rand::thread_rng();
        let selection_point: f64 = Uniform::new(0.0, total_reputation).sample(&mut rng);
        
        let mut cumulative_reputation = 0.0;
        for member in self.members.values() {
            cumulative_reputation += member.reputation;
            if cumulative_reputation >= selection_point {
                return Some(member.id.clone());
            }
        }

        None
    }

    // Decay the reputation of all members over time
    pub fn decay_reputation(&mut self) {
        let now = Utc::now();
        for member in self.members.values_mut() {
            let duration = now.signed_duration_since(member.last_decay).num_seconds() as f64;
            member.reputation *= (1.0 - DECAY_RATE).powf(duration);
            member.last_decay = now;
        }
    }

    // Validate a block based on consensus rules
    pub fn validate_block(&self, _block_index: u64) -> bool {
        // TODO: Implement validation logic based on consensus rules
        true
    }
}

// Proof of Contribution (PoC) consensus mechanism
pub struct PoCConsensus {
    pub consensus: Consensus,
    pub min_reputation: f64,
    pub vote_threshold: f64,
}

impl PoCConsensus {
    pub fn new(min_reputation: f64, vote_threshold: f64) -> Self {
        PoCConsensus {
            consensus: Consensus::new(),
            min_reputation,
            vote_threshold,
        }
    }

    // Additional PoC-specific methods can be added here
}