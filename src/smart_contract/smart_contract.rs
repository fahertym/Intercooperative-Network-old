// File: src/smart_contract/smart_contract.rs

use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Enum representing different types of smart contracts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractType {
    AssetTransfer,
    Proposal,
    ServiceAgreement,
    GovernanceVote,
    ResourceAllocation,
    IdentityVerification,
    CooperativeMembership,
    Custom(String),
}

// Enum representing the possible statuses of a smart contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractStatus {
    Pending,    // Contract is created but not yet active
    Active,     // Contract is currently in force
    Completed,  // Contract has been fulfilled
    Terminated, // Contract has been terminated before completion
}

// The main struct representing a smart contract
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SmartContract {
    pub id: String,                // Unique identifier for the contract
    pub contract_type: ContractType, // The type of the contract
    pub creator: String,           // The address of the contract creator
    pub created_at: DateTime<Utc>, // Timestamp of contract creation
    pub content: String,           // The actual code or logic of the contract
    pub status: ContractStatus,    // Current status of the contract
}

impl SmartContract {
    // Activate the contract, changing its status to Active
    pub fn activate(&mut self) {
        self.status = ContractStatus::Active;
    }

    // Mark the contract as completed
    pub fn complete(&mut self) {
        self.status = ContractStatus::Completed;
    }

    // Terminate the contract prematurely
    pub fn terminate(&mut self) {
        self.status = ContractStatus::Terminated;
    }

    // Execute the smart contract
    pub fn execute(&self, env: &mut ExecutionEnvironment) -> Result<String, String> {
        // Check if the contract is active before execution
        if self.status != ContractStatus::Active {
            return Err(format!("Contract is not active. Current status: {:?}", self.status));
        }

        // Execute the appropriate function based on the contract type
        match self.contract_type {
            ContractType::AssetTransfer => self.execute_asset_transfer(env),
            ContractType::Proposal => self.execute_proposal(env),
            ContractType::ServiceAgreement => self.execute_service_agreement(env),
            ContractType::GovernanceVote => self.execute_governance_vote(env),
            ContractType::ResourceAllocation => self.execute_resource_allocation(env),
            ContractType::IdentityVerification => self.execute_identity_verification(env),
            ContractType::CooperativeMembership => self.execute_cooperative_membership(env),
            ContractType::Custom(ref name) => self.execute_custom_contract(env, name),
        }
    }

    // Specific Execution Functions for Contract Types
    fn execute_asset_transfer(&self, env: &mut ExecutionEnvironment) -> Result<String, String> {
        let params: AssetTransferParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse asset transfer params: {}", e))?;

        // Check if the sender has sufficient balance
        if let Some(from_balance) = env.balances.get_mut(&params.from) {
            if let Some(amount) = from_balance.get_mut(&params.asset) {
                if *amount >= params.amount {
                    // Deduct from sender
                    *amount -= params.amount;
                    // Add to recipient
                    env.balances
                        .entry(params.to.clone())
                        .or_insert_with(HashMap::new)
                        .entry(params.asset.clone())
                        .and_modify(|e| *e += params.amount)
                        .or_insert(params.amount);
                    return Ok("Asset transfer successful".to_string());
                } else {
                    return Err("Insufficient balance".to_string());
                }
            } else {
                return Err("Asset not found in sender's balance".to_string());
            }
        } else {
            return Err("Sender not found".to_string());
        }
    }

    fn execute_proposal(&self, env: &mut ExecutionEnvironment) -> Result<String, String> {
        let params: ProposalParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse proposal params: {}", e))?;

        env.proposals.insert(self.id.clone(), params);
        Ok("Proposal added".to_string())
    }

    fn execute_service_agreement(&self, env: &mut ExecutionEnvironment) -> Result<String, String> {
        let params: ServiceAgreementParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse service agreement params: {}", e))?;

        env.service_agreements.insert(self.id.clone(), params);
        Ok("Service agreement added".to_string())
    }

    fn execute_governance_vote(&self, env: &mut ExecutionEnvironment) -> Result<String, String> {
        let params: GovernanceVoteParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse governance vote params: {}", e))?;

        env.votes.entry(params.proposal_id)
            .or_insert_with(Vec::new)
            .push((params.voter, params.vote));
        Ok("Vote recorded".to_string())
    }

    fn execute_resource_allocation(&self, env: &mut ExecutionEnvironment) -> Result<String, String> {
        let params: ResourceAllocationParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse resource allocation params: {}", e))?;

        env.resource_allocations.insert(self.id.clone(), params);
        Ok("Resource allocation added".to_string())
    }

    fn execute_identity_verification(&self, env: &mut ExecutionEnvironment) -> Result<String, String> {
        let params: IdentityVerificationParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse identity verification params: {}", e))?;

        env.identities.insert(params.user_id.clone(), params);
        Ok("Identity verified".to_string())
    }

    fn execute_cooperative_membership(&self, env: &mut ExecutionEnvironment) -> Result<String, String> {
        let params: CooperativeMembershipParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse cooperative membership params: {}", e))?;

        env.memberships.insert(params.user_id.clone(), params);
        Ok("Cooperative membership added".to_string())
    }

    fn execute_custom_contract(&self, env: &mut ExecutionEnvironment, name: &str) -> Result<String, String> {
        env.custom_contracts.insert(self.id.clone(), (name.to_string(), self.content.clone()));
        Ok("Custom contract executed".to_string())
    }
}

// The ExecutionEnvironment struct represents the context in which smart contracts are executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEnvironment {
    pub balances: HashMap<String, HashMap<String, f64>>, // User balances by asset type
    pub proposals: HashMap<String, ProposalParams>, // Active proposals
    pub service_agreements: HashMap<String, ServiceAgreementParams>, // Active service agreements
    pub resource_allocations: HashMap<String, ResourceAllocationParams>, // Resource allocations
    pub identities: HashMap<String, IdentityVerificationParams>, // Verified identities
    pub memberships: HashMap<String, CooperativeMembershipParams>, // Cooperative memberships
    pub votes: HashMap<String, Vec<(String, bool)>>, // Votes
    pub custom_contracts: HashMap<String, (String, String)>, // Custom contracts
}

impl ExecutionEnvironment {
    // Create a new ExecutionEnvironment
    pub fn new() -> Self {
        ExecutionEnvironment {
            balances: HashMap::new(),
            proposals: HashMap::new(),
            service_agreements: HashMap::new(),
            resource_allocations: HashMap::new(),
            identities: HashMap::new(),
            memberships: HashMap::new(),
            votes: HashMap::new(),
            custom_contracts: HashMap::new(),
        }
    }

    // Add balance to a user's account
    pub fn add_balance(&mut self, user: &str, asset: &str, amount: f64) {
        self.balances
            .entry(user.to_string())
            .or_insert_with(HashMap::new)
            .entry(asset.to_string())
            .and_modify(|e| *e += amount)
            .or_insert(amount);
    }

    // Get the balance of a user's account
    pub fn get_balance(&self, user: &str, asset: &str) -> f64 {
        self.balances
            .get(user)
            .and_then(|assets| assets.get(asset))
            .cloned()
            .unwrap_or(0.0)
    }

    // Tally votes for a proposal
    pub fn tally_votes(&self, proposal_id: &str) -> (usize, usize) {
        let votes = self.votes.get(proposal_id).cloned().unwrap_or_default();
        let (approve, reject): (Vec<_>, Vec<_>) = votes.into_iter().partition(|(_, vote)| *vote);
        (approve.len(), reject.len())
    }
}

// Parameter Structs for Different Contract Types
#[derive(Debug, Serialize, Deserialize)]
struct AssetTransferParams {
    from: String,
    to: String,
    asset: String,
    amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProposalParams {
    title: String,
    description: String,
    options: Vec<String>,
    #[serde(with = "duration_serde")]
    voting_period: std::time::Duration,
    quorum: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct GovernanceVoteParams {
    proposal_id: String,
    voter: String,
    vote: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAgreementParams {
    provider: String,
    consumer: String,
    service: String,
    terms: String,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceAllocationParams {
    resource: String,
    amount: f64,
    recipient: String,
    #[serde(with = "duration_serde")]
    duration: std::time::Duration,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdentityVerificationParams {
    user_id: String,
    verification_data: String,
    verification_method: String,
    expiration: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CooperativeMembershipParams {
    user_id: String,
    membership_type: String,
    join_date: DateTime<Utc>,
    #[serde(with = "duration_serde")]
    subscription_period: std::time::Duration,
}

// Utility Functions and Modules
#[allow(dead_code)]
pub fn parse_contract(input: &str) -> Result<SmartContract, String> {
    serde_json::from_str(input)
        .map_err(|e| format!("Failed to parse contract: {}", e))
}



// Module for serializing and deserializing Duration
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_contract_function() {
        let input = r#"{
            "id": "contract_123",
            "contract_type": "AssetTransfer",
            "creator": "Alice",
            "created_at": "2023-07-01T00:00:00Z",
            "content": "{\"from\": \"Alice\", \"to\": \"Bob\", \"asset\": \"ICN_TOKEN\", \"amount\": 100.0}",
            "status": "Pending"
        }"#;

        let contract = parse_contract(input).expect("Failed to parse contract");
        assert_eq!(contract.id, "contract_123");
        assert!(matches!(contract.contract_type, ContractType::AssetTransfer));
        assert_eq!(contract.creator, "Alice");
        assert_eq!(contract.status, ContractStatus::Pending);
    }

    #[test]
    fn test_execute_asset_transfer() {
        let mut env = ExecutionEnvironment::new();
        env.add_balance("Alice", "ICN_TOKEN", 1000.0);

        let mut contract = SmartContract {
            id: "contract_1".to_string(),
            contract_type: ContractType::AssetTransfer,
            creator: "Alice".to_string(),
            created_at: Utc::now(),
            content: r#"{"from": "Alice", "to": "Bob", "asset": "ICN_TOKEN", "amount": 100.0}"#.to_string(),
            status: ContractStatus::Pending,
        };

        contract.activate();
        
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.get_balance("Alice", "ICN_TOKEN"), 900.0);
        assert_eq!(env.get_balance("Bob", "ICN_TOKEN"), 100.0);
    }

    #[test]
    fn test_execute_proposal() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract {
            id: "contract_2".to_string(),
            contract_type: ContractType::Proposal,
            creator: "Charlie".to_string(),
            created_at: Utc::now(),
            content: r#"{"title": "New Project", "description": "Start a community garden", "options": ["Approve", "Reject"], "voting_period": 604800, "quorum": 0.5}"#.to_string(),
            status: ContractStatus::Pending,
        };

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.proposals.len(), 1);
        assert!(env.proposals.contains_key(&contract.id));
    }

    #[test]
    fn test_execute_governance_vote() {
        let mut env = ExecutionEnvironment::new();
        let proposal_id = "proposal_1".to_string();
        let mut contract = SmartContract {
            id: "contract_3".to_string(),
            contract_type: ContractType::GovernanceVote,
            creator: "Dave".to_string(),
            created_at: Utc::now(),
            content: format!(r#"{{"proposal_id": "{}", "voter": "Dave", "vote": true}}"#, proposal_id),
            status: ContractStatus::Pending,
        };

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.votes.len(), 1);
        assert_eq!(env.votes[&proposal_id], vec![("Dave".to_string(), true)]);
    }

    #[test]
    fn test_execute_service_agreement() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract {
            id: "contract_4".to_string(),
            contract_type: ContractType::ServiceAgreement,
            creator: "Eve".to_string(),
            created_at: Utc::now(),
            content: r#"{"provider": "Eve", "consumer": "Frank", "service": "Web Development", "terms": "Develop a website for 1000 ICN_TOKEN", "start_date": "2023-07-01T00:00:00Z", "end_date": "2023-08-01T00:00:00Z"}"#.to_string(),
            status: ContractStatus::Pending,
        };

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.service_agreements.len(), 1);
        assert!(env.service_agreements.contains_key(&contract.id));
    }

    #[test]
    fn test_execute_resource_allocation() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract {
            id: "contract_5".to_string(),
            contract_type: ContractType::ResourceAllocation,
            creator: "Grace".to_string(),
            created_at: Utc::now(),
            content: r#"{"resource": "Computing Power", "amount": 100.0, "recipient": "Research Team", "duration": 2592000}"#.to_string(),
            status: ContractStatus::Pending,
        };

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.resource_allocations.len(), 1);
        assert!(env.resource_allocations.contains_key(&contract.id));
    }

    #[test]
    fn test_execute_identity_verification() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract {
            id: "contract_6".to_string(),
            contract_type: ContractType::IdentityVerification,
            creator: "Henry".to_string(),
            created_at: Utc::now(),
            content: r#"{"user_id": "Henry", "verification_data": "Passport: AB123456", "verification_method": "Government ID", "expiration": "2025-07-01T00:00:00Z"}"#.to_string(),
            status: ContractStatus::Pending,
        };

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.identities.len(), 1);
        assert!(env.identities.contains_key("Henry"));
    }

    #[test]
    fn test_execute_cooperative_membership() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract {
            id: "contract_7".to_string(),
            contract_type: ContractType::CooperativeMembership,
            creator: "Ivy".to_string(),
            created_at: Utc::now(),
            content: r#"{"user_id": "Ivy", "membership_type": "Full Member", "join_date": "2023-07-01T00:00:00Z", "subscription_period": 31536000}"#.to_string(),
            status: ContractStatus::Pending,
        };

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.memberships.len(), 1);
        assert!(env.memberships.contains_key("Ivy"));
    }

    #[test]
    fn test_execute_custom_contract() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract {
            id: "contract_8".to_string(),
            contract_type: ContractType::Custom("DataSharing".to_string()),
            creator: "Jack".to_string(),
            created_at: Utc::now(),
            content: r#"{"data_provider": "Jack", "data_consumer": "Research Institute", "dataset": "Anonymous Health Records", "usage_terms": "Research purposes only", "compensation": 500}"#.to_string(),
            status: ContractStatus::Pending,
        };

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.custom_contracts.len(), 1);
        assert!(env.custom_contracts.contains_key(&contract.id));
    }

    #[test]
    fn test_contract_lifecycle() {
        let mut contract = SmartContract {
            id: "contract_9".to_string(),
            contract_type: ContractType::AssetTransfer,
            creator: "Alice".to_string(),
            created_at: Utc::now(),
            content: r#"{"from": "Alice", "to": "Bob", "asset": "ICN_TOKEN", "amount": 100.0}"#.to_string(),
            status: ContractStatus::Pending,
        };

        assert_eq!(contract.status, ContractStatus::Pending);

        contract.activate();
        assert_eq!(contract.status, ContractStatus::Active);

        contract.complete();
        assert_eq!(contract.status, ContractStatus::Completed);

        contract.terminate();
        assert_eq!(contract.status, ContractStatus::Terminated);
    }
}
