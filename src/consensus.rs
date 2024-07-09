// src/consensus.rs

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

type ReputationScores = HashMap<String, f64>;

#[derive(Clone, Debug)]
pub struct Vote {
    pub voter: String,
    pub in_favor: bool,
    pub weight: f64,
}

#[derive(Clone, Default)]
pub struct PoCConsensus {
    reputation_scores: ReputationScores,
    min_reputation_threshold: f64,
    max_reputation: f64,
    votes: HashMap<u64, Vec<Vote>>,
    vote_threshold: f64,
    last_decay: u64,
    decay_period: u64,
    decay_factor: f64,
    rehabilitation_rate: f64,
    slashing_severity: HashMap<String, f64>,
}

impl PoCConsensus {
    pub fn new(min_reputation_threshold: f64, vote_threshold: f64) -> Self {
        Self {
            reputation_scores: HashMap::new(),
            min_reputation_threshold,
            max_reputation: 100.0,
            votes: HashMap::new(),
            vote_threshold,
            last_decay: 0,
            decay_period: 86400,
            decay_factor: 0.95,
            rehabilitation_rate: 0.1,
            slashing_severity: HashMap::new(),
        }
    }

    pub fn set_vote_threshold(&mut self, new_threshold: f64) {
        self.vote_threshold = new_threshold.max(0.0).min(1.0);
    }

    pub fn add_member(&mut self, member_id: String) {
        self.reputation_scores.insert(member_id, self.min_reputation_threshold - 0.1);
    }

    pub fn is_eligible(&self, member_id: &str) -> bool {
        self.get_reputation(member_id).unwrap_or(0.0) >= self.min_reputation_threshold
    }

    pub fn update_reputation(&mut self, member_id: &str, delta: f64) {
        let old_reputation = self.reputation_scores.get(member_id).cloned().unwrap_or(0.0);
        let new_reputation = (old_reputation + delta).max(0.0).min(self.max_reputation);
        self.reputation_scores.insert(member_id.to_string(), new_reputation);
    }

    pub fn get_reputation(&self, member_id: &str) -> Option<f64> {
        self.reputation_scores.get(member_id).cloned()
    }

    pub fn select_proposer(&self) -> Option<String> {
        let eligible_members: Vec<_> = self.reputation_scores
            .iter()
            .filter(|(_, &score)| score >= self.min_reputation_threshold)
            .collect();

        if eligible_members.is_empty() {
            return None;
        }

        let total_reputation: f64 = eligible_members.iter().map(|(_, &score)| score).sum();
        let mut rng = rand::thread_rng();
        let selection_point: f64 = rng.gen_range(0.0, total_reputation);

        let mut cumulative_reputation = 0.0;
        for (member, &score) in eligible_members.into_iter().rev() {
            cumulative_reputation += score;
            if selection_point <= cumulative_reputation {
                return Some(member.clone());
            }
        }

        None
    }

    pub fn submit_vote(&mut self, block_index: u64, voter: String, in_favor: bool) {
        if self.is_eligible(&voter) {
            let weight = self.get_reputation(&voter).unwrap_or(0.0);
            self.votes.entry(block_index).or_insert_with(Vec::new).push(Vote { voter, in_favor, weight });
        }
    }

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

    pub fn finalize_block(&mut self, block_index: u64) {
        if let Some(votes) = self.votes.get(&block_index) {
            let voters_to_reward: Vec<String> = votes.iter().map(|v| v.voter.clone()).collect();
            for voter in voters_to_reward {
                self.update_reputation(&voter, 0.05);
            }
        }
        self.votes.remove(&block_index);
    }

    pub fn slash_reputation(&mut self, member_id: &str, offense: &str) {
        let slash_amount = self.slashing_severity.get(offense).cloned().unwrap_or(0.1);
        self.update_reputation(member_id, -slash_amount);
    }

    pub fn decay_reputations(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        if now - self.last_decay >= self.decay_period {
            for score in self.reputation_scores.values_mut() {
                *score *= self.decay_factor;
            }
            self.last_decay = now;
        }
    }

    pub fn rehabilitate_members(&mut self) {
        for (_, score) in self.reputation_scores.iter_mut() {
            if *score < self.min_reputation_threshold {
                *score += self.rehabilitation_rate;
                *score = score.min(self.min_reputation_threshold);
            }
        }
    }

    pub fn challenge_slashing(&mut self, member_id: &str, challenge_votes: usize) -> bool {
        let current_reputation = self.get_reputation(member_id).unwrap_or(0.0);
        let challenge_success_threshold = self.reputation_scores.len() / 2;

        if challenge_votes > challenge_success_threshold {
            let reputation_restore = self.max_reputation / 2.0;
            self.update_reputation(member_id, reputation_restore);
            println!("Slashing challenge successful for {}. Reputation restored by {}", member_id, reputation_restore);
            true
        } else {
            println!("Slashing challenge failed for {}. Reputation remains at {}", member_id, current_reputation);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poc_consensus() {
        let mut consensus = PoCConsensus::new(0.5, 0.66);
        
        consensus.add_member("Alice".to_string());
        consensus.add_member("Bob".to_string());
        consensus.add_member("Charlie".to_string());

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
        consensus.rehabilitate_members();

        assert!(consensus.challenge_slashing("Alice", 2));
    }
}