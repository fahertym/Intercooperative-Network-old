// Filename: src/democracy.rs

// ==================================================
// Imports
// ==================================================
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use crate::blockchain::Blockchain;

// ==================================================
// Enums and Structs for the Democratic System
// ==================================================

// Enum to represent different categories of proposals
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalCategory {
    Constitutional,
    Economic,
    Technical,
}

// Enum to represent the status of a proposal
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalStatus {
    Active,      // Proposal is currently being voted on
    Passed,      // Proposal has been approved by the required quorum
    Rejected,    // Proposal did not pass the voting process
    Implemented, // Proposal has been executed and its changes are in effect
}

// Enum to represent the type of a proposal
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ProposalType {
    Constitutional,    // Changes to the constitution or fundamental rules
    EconomicAdjustment, // Adjustments to economic policies
    NetworkUpgrade,     // Upgrades to the network or protocol
}

// Struct to represent a proposal
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proposal {
    pub id: String,                  // Unique identifier for the proposal
    pub title: String,               // Title of the proposal
    pub description: String,         // Detailed description of the proposal
    pub proposer: String,            // The person or entity proposing the change
    pub created_at: DateTime<Utc>,   // Timestamp when the proposal was created
    pub voting_ends_at: DateTime<Utc>, // Timestamp when the voting period ends
    pub status: ProposalStatus,      // Current status of the proposal
    pub proposal_type: ProposalType, // Type of the proposal
    pub category: ProposalCategory,  // Category of the proposal
    pub required_quorum: f64,        // The required quorum for the proposal to pass
    pub execution_timestamp: Option<DateTime<Utc>>, // When the proposal should be executed if passed
}

// Struct to represent a vote
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vote {
    pub voter: String,            // Unique identifier for the voter
    pub proposal_id: String,      // The proposal being voted on
    pub in_favor: bool,           // True if the vote is in favor, false otherwise
    pub weight: f64,              // Weight of the vote (could represent the voter's influence)
    pub timestamp: DateTime<Utc>, // Timestamp when the vote was cast
}

// Trait for executable proposals, allowing certain proposals to trigger changes in the blockchain
pub trait ExecutableProposal {
    fn execute(&self, blockchain: &mut Blockchain) -> Result<(), String>;
}

// Struct to represent a parameter change proposal
pub struct ParameterChangeProposal {
    pub parameter_name: String, // Name of the parameter to be changed
    pub new_value: String,      // New value for the parameter
}

// Implementation of the ExecutableProposal trait for ParameterChangeProposal
impl ExecutableProposal for ParameterChangeProposal {
    fn execute(&self, _blockchain: &mut Blockchain) -> Result<(), String> {
        // Implementation for changing a blockchain parameter
        // This is a placeholder and should be implemented based on your specific blockchain structure
        println!("Changing parameter {} to {}", self.parameter_name, self.new_value);
        Ok(())
    }
}

// Struct to represent the democratic system
pub struct DemocraticSystem {
    proposals: HashMap<String, Proposal>,                 // Map of proposal IDs to proposals
    votes: HashMap<String, Vec<Vote>>,                    // Map of proposal IDs to lists of votes
    executable_proposals: HashMap<String, Box<dyn ExecutableProposal>>, // Map of proposal IDs to executable proposals
}

// Implementation of the DemocraticSystem
impl DemocraticSystem {
    // Constructor for creating a new DemocraticSystem
    pub fn new() -> Self {
        DemocraticSystem {
            proposals: HashMap::new(),
            votes: HashMap::new(),
            executable_proposals: HashMap::new(),
        }
    }

    // Function to create a proposal
    pub fn create_proposal(
        &mut self, 
        title: String, 
        description: String, 
        proposer: String, 
        voting_duration: Duration, 
        proposal_type: ProposalType,
        category: ProposalCategory, 
        required_quorum: f64, 
        execution_timestamp: Option<DateTime<Utc>>
    ) -> String {
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

    // Function to vote on a proposal
    pub fn vote(
        &mut self, 
        voter: String, 
        proposal_id: String, 
        in_favor: bool, 
        weight: f64
    ) -> Result<(), String> {
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

    // Function to tally votes for a proposal
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

    // Function to get a specific proposal
    pub fn get_proposal(&self, proposal_id: &str) -> Option<&Proposal> {
        self.proposals.get(proposal_id)
    }

    // Function to get the votes for a specific proposal
    pub fn get_votes(&self, proposal_id: &str) -> Option<&Vec<Vote>> {
        self.votes.get(proposal_id)
    }

    // Function to list all active proposals
    pub fn list_active_proposals(&self) -> Vec<&Proposal> {
        self.proposals.values()
            .filter(|p| p.status == ProposalStatus::Active)
            .collect()
    }

    // Function to mark a proposal as implemented
    pub fn mark_as_implemented(&mut self, proposal_id: &str) -> Result<(), String> {
        let proposal = self.proposals.get_mut(proposal_id).ok_or("Proposal not found")?;
        
        if proposal.status != ProposalStatus::Passed {
            return Err("Proposal has not passed".to_string());
        }

        proposal.status = ProposalStatus::Implemented;
        Ok(())
    }

    // Function to add an executable proposal
    pub fn add_executable_proposal(&mut self, proposal_id: String, executable: Box<dyn ExecutableProposal>) {
        self.executable_proposals.insert(proposal_id, executable);
    }

    // Function to execute a passed proposal
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
}
