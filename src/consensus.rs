// consensus.rs

// This module implements the Proof-of-Contribution (PoC) consensus mechanism.
// It handles node management, reputation scores, voting, and block validation.

// We use several standard library collections and time management features.
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

// This struct represents a vote within the consensus mechanism.
#[derive(Clone, Debug)]
pub struct Vote {
    pub voter: String,    // The ID of the voter node
    pub in_favor: bool,   // Whether the vote is in favor or not
    pub weight: f64,      // The weight of the vote based on reputation
}

// The main PoCConsensus struct handles the state of the consensus mechanism.
#[derive(Clone, Default)]
pub struct PoCConsensus {
    pub reputation_scores: HashMap<String, f64>, // Stores the reputation scores of nodes
    pub min_reputation_threshold: f64,           // Minimum reputation required to participate
    pub max_reputation: f64,                     // Maximum possible reputation
    pub votes: HashMap<u64, Vec<Vote>>,          // Records votes for each block
    pub vote_threshold: f64,                     // Threshold for a block to be considered valid
    pub last_decay: u64,                         // Timestamp of the last reputation decay
    pub decay_period: u64,                       // How often reputations should decay
    pub decay_factor: f64,                       // Factor by which reputations decay
    pub rehabilitation_rate: f64,                // Rate at which low reputations are rehabilitated
    pub slashing_severity: HashMap<String, f64>, // Severity of reputation slashing for offenses
}

impl PoCConsensus {
    // Create a new PoCConsensus instance with initial parameters.
    pub fn new(min_reputation_threshold: f64, vote_threshold: f64) -> Self {
        Self {
            reputation_scores: HashMap::new(),
            min_reputation_threshold,
            max_reputation: 100.0,
            votes: HashMap::new(),
            vote_threshold,
            last_decay: 0,
            decay_period: 86400, // Default to 1 day (86400 seconds)
            decay_factor: 0.95,
            rehabilitation_rate: 0.1,
            slashing_severity: HashMap::new(),
        }
    }

    // Set the vote threshold, ensuring it's between 0 and 1.
    pub fn set_vote_threshold(&mut self, new_threshold: f64) {
        self.vote_threshold = new_threshold.max(0.0).min(1.0);
    }

    // Add a new node to the consensus mechanism.
    pub fn add_node(&mut self, node_id: String) {
        self.reputation_scores.insert(node_id, self.min_reputation_threshold - 0.1);
    }

    // Check if a node is eligible to participate based on their reputation.
    pub fn is_eligible(&self, node_id: &str) -> bool {
        self.get_reputation(node_id).unwrap_or(0.0) >= self.min_reputation_threshold
    }

    // Update a node's reputation based on their contribution.
    pub fn update_reputation(&mut self, node_id: &str, delta: f64) {
        let old_reputation = self.reputation_scores.get(node_id).cloned().unwrap_or(0.0);
        let new_reputation = (old_reputation + delta).max(0.0).min(self.max_reputation);
        self.reputation_scores.insert(node_id.to_string(), new_reputation);
    }

    // Get the reputation of a specific node.
    pub fn get_reputation(&self, node_id: &str) -> Option<f64> {
        self.reputation_scores.get(node_id).cloned()
    }

    // Select a proposer for the next block based on reputation scores.
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
            if selection_point <= cumulative_reputation {
                return Some(node.clone());
            }
        }

        None
    }

    // Submit a vote for a specific block.
    pub fn submit_vote(&mut self, block_index: u64, voter: String, in_favor: bool) {
        if self.is_eligible(&voter) {
            let weight = self.get_reputation(&voter).unwrap_or(0.0);
            self.votes.entry(block_index).or_insert_with(Vec::new).push(Vote { voter, in_favor, weight });
        }
    }

    // Check if a block is valid based on the votes it received.
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

    // Finalize a block and update reputations based on votes.
    pub fn finalize_block(&mut self, block_index: u64) {
        if let Some(votes) = self.votes.get(&block_index) {
            let voters_to_reward: Vec<String> = votes.iter().map(|v| v.voter.clone()).collect();
            for voter in voters_to_reward {
                self.update_reputation(&voter, 0.05);
            }
        }
        self.votes.remove(&block_index);
    }

    // Slash the reputation of a node for an offense.
    pub fn slash_reputation(&mut self, node_id: &str, offense: &str) {
        let slash_amount = self.slashing_severity.get(offense).cloned().unwrap_or(0.1);
        self.update_reputation(node_id, -slash_amount);
    }

    // Decay reputations periodically.
    pub fn decay_reputations(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if now - self.last_decay >= self.decay_period {
            for score in self.reputation_scores.values_mut() {
                *score *= self.decay_factor;
            }
            self.last_decay = now;
        }
    }

    // Rehabilitate nodes with low reputations gradually.
    pub fn rehabilitate_nodes(&mut self) {
        for (_, score) in self.reputation_scores.iter_mut() {
            if *score < self.min_reputation_threshold {
                *score += self.rehabilitation_rate;
                *score = score.min(self.min_reputation_threshold);
            }
        }
    }

    // Challenge a reputation slashing event.
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

// Unit tests for PoCConsensus functionality.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poc_consensus() {
        let mut consensus = PoCConsensus::new(0.5, 0.66);
        
        consensus.add_node("Alice".to_string());
        consensus.add_node("Bob".to_string());
        consensus.add_node("Charlie".to_string());

        assert_eq!(consensus.get_reputation("Alice"), Some(0.4));
        assert_eq!(consensus.get_reputation("Bob"), Some(0.4));
        assert_eq!(consensus.get_reputation("Charlie"), Some(0.4));

        consensus.update_reputation("Alice", 0.2);
        consensus.update_reputation("Bob", 0.1);

        assert!(consensus.is_eligible("Alice"));
        assert!(consensus.is_eligible("Bob"));
        assert!(!consensus.is_eligible("Charlie"));

        consensus.submit_vote(1, "Alice".to_string(), true);
        consensus.submit_vote(1, "Bob".to_string(), true);
        consensus.submit_vote(1, "Charlie".to_string(), false);

        assert!(consensus.is_block_valid(1));

        consensus.finalize_block(1);

        assert!(consensus.get_reputation("Alice").unwrap() > 0.6);
        assert!(consensus.get_reputation("Bob").unwrap() > 0.5);
        assert!(consensus.get_reputation("Charlie").unwrap() < 0.5);

        consensus.slash_reputation("Alice", "minor_offense");
        assert!(consensus.get_reputation("Alice").unwrap() < 0.6);

        consensus.decay_reputations();
        consensus.rehabilitate_nodes();

        assert!(consensus.challenge_slashing("Alice", 2));
    }
}
