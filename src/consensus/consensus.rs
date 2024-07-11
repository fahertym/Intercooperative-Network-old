// Filename: src/consensus.rs

// ===============================================
// Consensus Mechanism Implementation
// ===============================================
// This file implements the Proof of Contribution (PoC) consensus mechanism
// for our blockchain. It defines how nodes in the network reach agreement
// on the state of the blockchain and which blocks are valid.
//
// Key concepts:
// - Proof of Contribution (PoC): A consensus mechanism that rewards nodes
//   based on their contributions to the network
// - Reputation: A measure of a node's trustworthiness and contributions
// - Voting: The process by which nodes agree on the validity of new blocks
// - Slashing: Punishment for malicious or faulty behavior
// - Rehabilitation: The process of regaining reputation after being slashed

use std::collections::HashMap;
use std::time::{Duration, Instant};
use rand::Rng;

// ===============================================
// Type Aliases and Constants
// ===============================================

type ReputationScores = HashMap<String, f64>;

const DEFAULT_MIN_REPUTATION_THRESHOLD: f64 = 0.5;
const DEFAULT_MAX_REPUTATION: f64 = 100.0;
const DEFAULT_VOTE_THRESHOLD: f64 = 0.66;
const DEFAULT_DECAY_PERIOD: Duration = Duration::from_secs(86400); // 1 day
const DEFAULT_DECAY_FACTOR: f64 = 0.95;
const DEFAULT_REHABILITATION_RATE: f64 = 0.1;

// ===============================================
// Vote Struct
// ===============================================
// Represents a single vote in the consensus process

#[derive(Clone, Debug)]
pub struct Vote {
    pub voter: String,    // The ID of the voting node
    pub in_favor: bool,   // Whether the vote is in favor of the block
    pub weight: f64,      // The weight of the vote, based on the voter's reputation
}

// ===============================================
// PoCConsensus Struct
// ===============================================
// The main structure implementing the Proof of Contribution consensus mechanism

#[derive(Clone, Default)]
pub struct PoCConsensus {
    pub reputation_scores: ReputationScores,   // Reputation scores of all nodes
    pub min_reputation_threshold: f64,         // Minimum reputation required to participate
    pub max_reputation: f64,                   // Maximum possible reputation score
    pub votes: HashMap<u64, Vec<Vote>>,        // Votes for each block (key is block index)
    pub vote_threshold: f64,                   // Threshold for a block to be considered valid
    pub last_decay: Instant,                   // Timestamp of the last reputation decay
    pub decay_period: Duration,                // How often reputations should decay
    pub decay_factor: f64,                     // Factor by which reputations decay
    pub rehabilitation_rate: f64,              // Rate at which low reputations are rehabilitated
    pub slashing_severity: HashMap<String, f64>, // Severity of slashing for different offenses
}

impl PoCConsensus {
    // Create a new PoCConsensus instance with default or custom parameters
    pub fn new(min_reputation_threshold: Option<f64>, vote_threshold: Option<f64>) -> Self {
        Self {
            reputation_scores: HashMap::new(),
            min_reputation_threshold: min_reputation_threshold.unwrap_or(DEFAULT_MIN_REPUTATION_THRESHOLD),
            max_reputation: DEFAULT_MAX_REPUTATION,
            votes: HashMap::new(),
            vote_threshold: vote_threshold.unwrap_or(DEFAULT_VOTE_THRESHOLD),
            last_decay: Instant::now(),
            decay_period: DEFAULT_DECAY_PERIOD,
            decay_factor: DEFAULT_DECAY_FACTOR,
            rehabilitation_rate: DEFAULT_REHABILITATION_RATE,
            slashing_severity: HashMap::new(),
        }
    }

    // Set the vote threshold, ensuring it's between 0 and 1
    pub fn set_vote_threshold(&mut self, new_threshold: f64) {
        self.vote_threshold = new_threshold.max(0.0).min(1.0);
    }

    // Add a new node to the consensus mechanism
    pub fn add_node(&mut self, node_id: String) {
        self.reputation_scores.insert(node_id, self.min_reputation_threshold - 0.1);
    }

    // Check if a node is eligible to participate in consensus
    pub fn is_eligible(&self, node_id: &str) -> bool {
        self.get_reputation(node_id).unwrap_or(0.0) >= self.min_reputation_threshold
    }

    // Update a node's reputation score
    pub fn update_reputation(&mut self, node_id: &str, delta: f64) {
        let old_reputation = self.reputation_scores.get(node_id).cloned().unwrap_or(0.0);
        let new_reputation = (old_reputation + delta).max(0.0).min(self.max_reputation);
        self.reputation_scores.insert(node_id.to_string(), new_reputation);
    }

    // Get a node's current reputation score
    pub fn get_reputation(&self, node_id: &str) -> Option<f64> {
        self.reputation_scores.get(node_id).cloned()
    }

    // Select a proposer for the next block based on reputation scores
    pub fn select_proposer(&self) -> Option<String> {
        let eligible_nodes: Vec<_> = self.reputation_scores
            .iter()
            .filter(|(_, &score)| score >= self.min_reputation_threshold)
            .collect();

        if eligible_nodes.is_empty() {
            return None;
        }

        let total_reputation: f64 = eligible_nodes.iter().map(|(_, &score)| score).sum();
        let mut rng = rand::thread_rng();
        let selection_point: f64 = rng.gen_range(0.0..total_reputation);

        let mut cumulative_reputation = 0.0;
        for (node, &score) in eligible_nodes {
            cumulative_reputation += score;
            if cumulative_reputation >= selection_point {
                return Some(node.clone());
            }
        }

        None
    }

    // Submit a vote for a block
    pub fn submit_vote(&mut self, block_index: u64, voter: String, in_favor: bool) {
        if self.is_eligible(&voter) {
            let weight = self.get_reputation(&voter).unwrap_or(0.0);
            self.votes.entry(block_index).or_insert_with(Vec::new).push(Vote { voter, in_favor, weight });
        }
    }

    // Check if a block is valid based on the votes it has received
    pub fn is_block_valid(&self, block_index: u64) -> bool {
        if let Some(votes) = self.votes.get(&block_index) {
            let total_weight: f64 = votes.iter().map(|v| v.weight).sum();
            let weighted_votes_in_favor: f64 = votes.iter()
                .filter(|v| v.in_favor)
                .map(|v| v.weight)
                .sum();

            let is_valid = weighted_votes_in_favor / total_weight >= self.vote_threshold;
            println!("Block {} validity check: {} (votes in favor: {:.2}, total votes: {:.2}, threshold: {:.2})",
                     block_index, is_valid, weighted_votes_in_favor, total_weight, self.vote_threshold);
            is_valid
        } else {
            println!("Block {} has no votes", block_index);
            false
        }
    }

    // Finalize a block by rewarding voters and clearing the votes
    pub fn finalize_block(&mut self, block_index: u64) {
        if let Some(votes) = self.votes.get(&block_index) {
            let voters_to_reward: Vec<String> = votes.iter().map(|v| v.voter.clone()).collect();
            for voter in voters_to_reward {
                self.update_reputation(&voter, 0.05);
            }
        }
        self.votes.remove(&block_index);
    }

    // Slash a node's reputation for an offense
    pub fn slash_reputation(&mut self, node_id: &str, offense: &str) {
        let slash_amount = self.slashing_severity.get(offense).cloned().unwrap_or(0.1);
        self.update_reputation(node_id, -slash_amount);
    }

    // Decay reputations over time to prevent stagnation
    pub fn decay_reputations(&mut self) {
        if self.last_decay.elapsed() >= self.decay_period {
            for score in self.reputation_scores.values_mut() {
                *score *= self.decay_factor;
            }
            self.last_decay = Instant::now();
        }
    }

    // Rehabilitate nodes with low reputation
    pub fn rehabilitate_nodes(&mut self) {
        for (_, score) in self.reputation_scores.iter_mut() {
            if *score < self.min_reputation_threshold {
                *score += self.rehabilitation_rate;
                *score = score.min(self.min_reputation_threshold);
            }
        }
    }

    // Challenge a slashing decision through a voting process
    pub fn challenge_slashing(&mut self, node_id: &str, challenge_votes: usize) -> bool {
        let current_reputation = self.get_reputation(node_id).unwrap_or(0.0);
        let challenge_success_threshold = self.reputation_scores.len() / 2;

        if challenge_votes > challenge_success_threshold {
            let reputation_restore = self.max_reputation / 2.0;
            self.update_reputation(node_id, reputation_restore);
            println!("Slashing challenge successful for {}. Reputation restored by {}", node_id, reputation_restore);
            true
        } else {
            println!("Slashing challenge failed for {}. Reputation remains at {}", node_id, current_reputation);
            false
        }
    }
}

// ===============================================
// Tests
// ===============================================
// Unit tests to verify the functionality of our consensus mechanism

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poc_consensus() {
        let mut consensus = PoCConsensus::new(Some(0.5), Some(0.66));
        
        // Test adding nodes
        consensus.add_node("Alice".to_string());
        consensus.add_node("Bob".to_string());
        consensus.add_node("Charlie".to_string());

        assert_eq!(consensus.get_reputation("Alice"), Some(0.4));
        assert_eq!(consensus.get_reputation("Bob"), Some(0.4));
        assert_eq!(consensus.get_reputation("Charlie"), Some(0.4));

        // Test updating reputations
        consensus.update_reputation("Alice", 0.2);
        consensus.update_reputation("Bob", 0.1);

        assert!(consensus.is_eligible("Alice"));
        assert!(consensus.is_eligible("Bob"));
        assert!(!consensus.is_eligible("Charlie"));

        // Test voting
        consensus.submit_vote(1, "Alice".to_string(), true);
        consensus.submit_vote(1, "Bob".to_string(), true);
        consensus.submit_vote(1, "Charlie".to_string(), false);

        assert!(consensus.is_block_valid(1));

        // Test block finalization
        consensus.finalize_block(1);

        assert!(consensus.get_reputation("Alice").unwrap() > 0.6);
        assert!(consensus.get_reputation("Bob").unwrap() > 0.5);
        assert!(consensus.get_reputation("Charlie").unwrap() < 0.5);

        // Test slashing
        consensus.slash_reputation("Alice", "minor_offense");
        assert!(consensus.get_reputation("Alice").unwrap() < 0.6);

        // Test decay and rehabilitation
        consensus.decay_reputations();
        consensus.rehabilitate_nodes();

        // Test slashing challenge
        assert!(consensus.challenge_slashing("Alice", 2));
    }
}

// ===============================================
// End of File
// ===============================================
// This concludes the implementation of our Proof of Contribution consensus
// mechanism. It provides the core functionality for maintaining node
// reputations, voting on blocks, and ensuring the integrity of the blockchain.