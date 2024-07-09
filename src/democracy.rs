use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use crate::blockchain::Blockchain;

// NEW CODE BLOCK 1: START
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalCategory {
    Constitutional,
    Economic,
    Technical,
}
// NEW CODE BLOCK 1: END

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proposal {
    pub id: String,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub created_at: DateTime<Utc>,
    pub voting_ends_at: DateTime<Utc>,
    pub status: ProposalStatus,
    pub proposal_type: ProposalType,
    // NEW CODE BLOCK 2: START
    pub category: ProposalCategory,
    pub required_quorum: f64,
    pub execution_timestamp: Option<DateTime<Utc>>,
    // NEW CODE BLOCK 2: END
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Implemented,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalType {
    Constitutional,
    EconomicAdjustment,
    NetworkUpgrade,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: String,
    pub in_favor: bool,
    pub weight: f64,
    pub timestamp: DateTime<Utc>,
}

// NEW CODE BLOCK 3: START
pub trait ExecutableProposal {
    fn execute(&self, blockchain: &mut Blockchain) -> Result<(), String>;
}

pub struct ParameterChangeProposal {
    pub parameter_name: String,
    pub new_value: String,
}

impl ExecutableProposal for ParameterChangeProposal {
    fn execute(&self, blockchain: &mut Blockchain) -> Result<(), String> {
        // Implementation for changing a blockchain parameter
        // This is a placeholder and should be implemented based on your specific blockchain structure
        println!("Changing parameter {} to {}", self.parameter_name, self.new_value);
        Ok(())
    }
}
// NEW CODE BLOCK 3: END

pub struct DemocraticSystem {
    proposals: HashMap<String, Proposal>,
    votes: HashMap<String, Vec<Vote>>,
    // NEW CODE BLOCK 4: START
    executable_proposals: HashMap<String, Box<dyn ExecutableProposal>>,
    // NEW CODE BLOCK 4: END
}

impl DemocraticSystem {
    pub fn new() -> Self {
        DemocraticSystem {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            // NEW CODE BLOCK 5: START
            executable_proposals: HashMap::new(),
            // NEW CODE BLOCK 5: END
        }
    }

    // NEW CODE BLOCK 6: START
    pub fn create_proposal(&mut self, title: String, description: String, proposer: String, 
                           voting_duration: Duration, proposal_type: ProposalType,
                           category: ProposalCategory, required_quorum: f64, 
                           execution_timestamp: Option<DateTime<Utc>>) -> String {
        let id = format!("prop_{}", Utc::now().timestamp());
        let proposal = Proposal {
            id: id.clone(),
            title,
            description,
            proposer,
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + voting_duration,
            status: ProposalStatus::Active,
            proposal_type,
            category,
            required_quorum,
            execution_timestamp,
        };
        self.proposals.insert(id.clone(), proposal);
        id
    }
    // NEW CODE BLOCK 6: END

    pub fn vote(&mut self, voter: String, proposal_id: String, in_favor: bool, weight: f64) -> Result<(), String> {
        let proposal = self.proposals.get(&proposal_id).ok_or("Proposal not found")?;
        
        if proposal.status != ProposalStatus::Active {
            return Err("Voting is not active for this proposal".to_string());
        }

        if Utc::now() > proposal.voting_ends_at {
            return Err("Voting period has ended".to_string());
        }

        let vote = Vote {
            voter,
            proposal_id: proposal_id.clone(),
            in_favor,
            weight,
            timestamp: Utc::now(),
        };

        self.votes.entry(proposal_id).or_insert_with(Vec::new).push(vote);
        Ok(())
    }

    // NEW CODE BLOCK 7: START
    pub fn tally_votes(&mut self, proposal_id: &str) -> Result<(), String> {
        let proposal = self.proposals.get_mut(proposal_id).ok_or("Proposal not found")?;
        
        if proposal.status != ProposalStatus::Active {
            return Err("Proposal is not active".to_string());
        }

        if Utc::now() < proposal.voting_ends_at {
            return Err("Voting period has not ended yet".to_string());
        }

        let votes = self.votes.get(proposal_id).ok_or("No votes found for this proposal")?;
        
        let total_weight: f64 = votes.iter().map(|v| v.weight).sum();
        let weight_in_favor: f64 = votes.iter().filter(|v| v.in_favor).map(|v| v.weight).sum();

        if total_weight < proposal.required_quorum {
            proposal.status = ProposalStatus::Rejected;
            return Ok(());
        }

        if weight_in_favor / total_weight > 0.5 {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        Ok(())
    }
    // NEW CODE BLOCK 7: END

    pub fn get_proposal(&self, proposal_id: &str) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    pub fn get_votes(&self, proposal_id: &str) -> Option<&Vec<Vote>> {
        self.votes.get(proposal_id)
    }

    pub fn list_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values()
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }

    pub fn mark_as_implemented(&mut self, proposal_id: &str) -> Result<(), String> {
        let proposal = self.proposals.get_mut(proposal_id).ok_or("Proposal not found")?;
        
        if proposal.status != ProposalStatus::Passed {
            return Err("Proposal has not passed".to_string());
        }

        proposal.status = ProposalStatus::Implemented;
        Ok(())
    }

    // NEW CODE BLOCK 8: START
    pub fn add_executable_proposal(&mut self, proposal_id: String, executable: Box<dyn ExecutableProposal>) {
        self.executable_proposals.insert(proposal_id, executable);
    }

    pub fn execute_proposal(&mut self, proposal_id: &str, blockchain: &mut Blockchain) -> Result<(), String> {
        let proposal = self.proposals.get(proposal_id).ok_or("Proposal not found")?;
        
        if proposal.status != ProposalStatus::Passed {
            return Err("Proposal has not passed".to_string());
        }

        if let Some(executable) = self.executable_proposals.get(proposal_id) {
            executable.execute(blockchain)?;
            self.mark_as_implemented(proposal_id)?;
            Ok(())
        } else {
            Err("No executable found for this proposal".to_string())
        }
    }
    // NEW CODE BLOCK 8: END
}