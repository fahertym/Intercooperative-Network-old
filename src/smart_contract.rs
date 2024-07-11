// Filename: smart_contract.rs

// =================================================
// Overview
// =================================================
// This file defines the SmartContract struct and its associated functionality.
// Smart contracts are self-executing contracts with the terms of the agreement
// directly written into code. This file also defines the ExecutionEnvironment struct,
// which provides the context and state for executing smart contracts.

// =================================================
// Imports
// =================================================

use std::collections::HashMap; // HashMap is used to store balances, proposals, votes, and other contract-related data.
use serde::{Serialize, Deserialize}; // Serde is used for serializing and deserializing contract data.
use chrono::{DateTime, Utc}; // Chrono is used for handling date and time.

// =================================================
// Enums and Structs for Smart Contracts
// =================================================

// Define the SmartContract struct which represents a smart contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContract {
    pub id: String, // Unique identifier for the contract.
    pub contract_type: ContractType, // The type of the contract (e.g., AssetTransfer, Proposal).
    pub creator: String, // The creator of the contract.
    pub created_at: DateTime<Utc>, // The creation timestamp of the contract.
    pub content: String, // The content of the contract in JSON format.
    pub status: ContractStatus, // The current status of the contract (e.g., Pending, Active).
}

// Define the ContractType enum which represents different types of contracts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractType {
    AssetTransfer, // Contract for transferring assets.
    Proposal, // Contract for making proposals.
    ServiceAgreement, // Contract for service agreements.
    GovernanceVote, // Contract for governance voting.
    ResourceAllocation, // Contract for allocating resources.
    IdentityVerification, // Contract for identity verification.
    CooperativeMembership, // Contract for cooperative membership.
    Custom(String), // Custom contract with a specific name.
}

// Define the ContractStatus enum which represents the status of the contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractStatus {
    Pending, // Contract is pending and not yet active.
    Active, // Contract is active and can be executed.
    Completed, // Contract has been completed.
    Terminated, // Contract has been terminated.
}

// =================================================
// Implementation of SmartContract
// =================================================

impl SmartContract {
    // Constructor for creating a new SmartContract.
    pub fn new(contract_type: ContractType, creator: String, content: String) -> Self {
        SmartContract {
            id: format!("contract_{}", Utc::now().timestamp()), // Generate a unique ID using the current timestamp.
            contract_type, // Set the contract type.
            creator, // Set the creator of the contract.
            created_at: Utc::now(), // Set the creation timestamp.
            content, // Set the content of the contract.
            status: ContractStatus::Pending, // Initialize the status as Pending.
        }
    }

    // Method to activate the contract.
    pub fn activate(&mut self) {
        self.status = ContractStatus::Active;
    }

    // Method to complete the contract.
    pub fn complete(&mut self) {
        self.status = ContractStatus::Completed;
    }

    // Method to terminate the contract.
    pub fn terminate(&mut self) {
        self.status = ContractStatus::Terminated;
    }

    // Main execute function that dispatches to specific execution functions based on the contract type.
    pub fn execute(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        if self.status != ContractStatus::Active {
            return Err(format!("Contract is not active. Current status: {:?}", self.status));
        }

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

    // ================================================
    // Specific Execution Functions for Contract Types
    // ================================================

    // Execute an asset transfer contract.
    fn execute_asset_transfer(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: AssetTransferParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse asset transfer params: {}", e))?;

        if let Some(from_balance) = env.balances.get_mut(&params.from) {
            if let Some(amount) = from_balance.get_mut(&params.asset) {
                if *amount >= params.amount {
                    *amount -= params.amount;
                    env.balances
                        .entry(params.to.clone())
                        .or_insert_with(HashMap::new)
                        .entry(params.asset.clone())
                        .and_modify(|e| *e += params.amount)
                        .or_insert(params.amount);
                    Ok(())
                } else {
                    Err("Insufficient balance".to_string())
                }
            } else {
                Err("Asset not found in sender's balance".to_string())
            }
        } else {
            Err("Sender not found".to_string())
        }
    }

    // Execute a proposal contract.
    fn execute_proposal(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: ProposalParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse proposal params: {}", e))?;

        env.proposals.insert(self.id.clone(), params);
        Ok(())
    }

    // Execute a service agreement contract.
    fn execute_service_agreement(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: ServiceAgreementParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse service agreement params: {}", e))?;

        env.service_agreements.insert(self.id.clone(), params);
        Ok(())
    }

    // Execute a governance vote contract.
    fn execute_governance_vote(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: GovernanceVoteParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse governance vote params: {}", e))?;

        env.votes.entry(params.proposal_id)
            .or_insert_with(Vec::new)
            .push((params.voter, params.vote));
        Ok(())
    }

    // Execute a resource allocation contract.
    fn execute_resource_allocation(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: ResourceAllocationParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse resource allocation params: {}", e))?;

        env.resource_allocations.insert(self.id.clone(), params);
        Ok(())
    }

    // Execute an identity verification contract.
    fn execute_identity_verification(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: IdentityVerificationParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse identity verification params: {}", e))?;

        env.identities.insert(params.user_id.clone(), params);
        Ok(())
    }

    // Execute a cooperative membership contract.
    fn execute_cooperative_membership(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: CooperativeMembershipParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse cooperative membership params: {}", e))?;

        env.memberships.insert(params.user_id.clone(), params);
        Ok(())
    }

    // Execute a custom contract.
    fn execute_custom_contract(&self, env: &mut ExecutionEnvironment, name: &str) -> Result<(), String> {
        env.custom_contracts.insert(self.id.clone(), (name.to_string(), self.content.clone()));
        Ok(())
    }
}

// =================================================
// Structs for Execution Environment and Params
// =================================================

// Define the ExecutionEnvironment struct which represents the context and state for executing smart contracts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEnvironment {
    pub balances: HashMap<String, HashMap<String, f64>>, // Balances of different users.
    pub proposals: HashMap<String, ProposalParams>, // Proposals in the environment.
    pub votes: HashMap<String, Vec<(String, bool)>>, // Votes for different proposals.
    pub service_agreements: HashMap<String, ServiceAgreementParams>, // Service agreements.
    pub resource_allocations: HashMap<String, ResourceAllocationParams>, // Resource allocations.
    pub identities: HashMap<String, IdentityVerificationParams>, // Identity verifications.
    pub memberships: HashMap<String, CooperativeMembershipParams>, // Cooperative memberships.
    pub custom_contracts: HashMap<String, (String, String)>, // Custom contracts.
}

// Implementation for the ExecutionEnvironment.
impl ExecutionEnvironment {
    // Constructor for creating a new ExecutionEnvironment.
    pub fn new() -> Self {
        ExecutionEnvironment {
            balances: HashMap::new(),
            proposals: HashMap::new(),
            votes: HashMap::new(),
            service_agreements: HashMap::new(),
            resource_allocations: HashMap::new(),
            identities: HashMap::new(),
            memberships: HashMap::new(),
            custom_contracts: HashMap::new(),
        }
    }

    // Add balance to a user's account.
    pub fn add_balance(&mut self, user: &str, asset: &str, amount: f64) {
        self.balances
            .entry(user.to_string())
            .or_insert_with(HashMap::new)
            .entry(asset.to_string())
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);
    }

    // Get the balance of a user's account.
    pub fn get_balance(&self, user: &str, asset: &str) -> f64 {
        self.balances
            .get(user)
            .and_then(|assets| assets.get(asset))
            .cloned()
            .unwrap_or(0.0)
    }

    // Tally the votes for a given proposal.
    pub fn tally_votes(&self, proposal_id: &str) -> (usize, usize) {
        let votes = self.votes.get(proposal_id).cloned().unwrap_or_default();
        let (approve, reject): (Vec<_>, Vec<_>) = votes.into_iter().partition(|(_, vote)| *vote);
        (approve.len(), reject.len())
    }
}

// =================================================
// Structs for Specific Params
// =================================================

// Define the parameters for asset transfer contracts.
#[derive(Debug, Serialize, Deserialize)]
struct AssetTransferParams {
    from: String, // Sender's account.
    to: String, // Receiver's account.
    asset: String, // The asset being transferred.
    amount: f64, // The amount of the asset being transferred.
}

// Define the parameters for proposal contracts.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProposalParams {
    title: String, // The title of the proposal.
    description: String, // The description of the proposal.
    options: Vec<String>, // The options available in the proposal.
    #[serde(with = "duration_serde")]
    voting_period: std::time::Duration, // The duration of the voting period.
    quorum: f64, // The quorum required for the proposal to pass.
}

// Define the parameters for governance vote contracts.
#[derive(Debug, Serialize, Deserialize)]
struct GovernanceVoteParams {
    proposal_id: String, // The ID of the proposal being voted on.
    voter: String, // The voter.
    vote: bool, // The vote (true for approve, false for reject).
}

// Define the parameters for service agreement contracts.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAgreementParams {
    provider: String, // The service provider.
    consumer: String, // The service consumer.
    service: String, // The service being provided.
    terms: String, // The terms of the service agreement.
    start_date: DateTime<Utc>, // The start date of the service agreement.
    end_date: DateTime<Utc>, // The end date of the service agreement.
}

// Define the parameters for resource allocation contracts.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceAllocationParams {
    resource: String, // The resource being allocated.
    amount: f64, // The amount of the resource being allocated.
    recipient: String, // The recipient of the resource.
    #[serde(with = "duration_serde")]
    duration: std::time::Duration, // The duration for which the resource is allocated.
}

// Define the parameters for identity verification contracts.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdentityVerificationParams {
    user_id: String, // The ID of the user being verified.
    verification_data: String, // The data used for verification.
    verification_method: String, // The method of verification.
    expiration: DateTime<Utc>, // The expiration date of the verification.
}

// Define the parameters for cooperative membership contracts.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CooperativeMembershipParams {
    user_id: String, // The ID of the user becoming a member.
    membership_type: String, // The type of membership.
    join_date: DateTime<Utc>, // The join date.
    #[serde(with = "duration_serde")]
    subscription_period: std::time::Duration, // The subscription period.
}

// =================================================
// Utility Functions and Modules
// =================================================

// Function to parse a smart contract from a JSON string.
pub fn parse_contract(input: &str) -> Result<SmartContract, String> {
    let contract: SmartContract = serde_json::from_str(input)
        .map_err(|e| format!("Failed to parse contract: {}", e))?;
    Ok(contract)
}

// Module for handling serialization and deserialization of durations.
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    // Serialize a duration to seconds.
    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    // Deserialize a duration from seconds.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}

// =================================================
// Unit Tests
// =================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_contract() {
        let input = r#"{
            "id": "contract_123",
            "contract_type": "AssetTransfer",
            "creator": "Alice",
            "created_at": "2023-07-01T00:00:00Z",
            "content": "{\"from\": \"Alice\", \"to\": \"Bob\", \"asset\": \"ICN_TOKEN\", \"amount\": 100.0}",
            "status": "Pending"
        }"#;

        let contract = parse_contract(input).unwrap();
        assert!(matches!(contract.contract_type, ContractType::AssetTransfer));
        assert_eq!(contract.creator, "Alice");
    }

    #[test]
    fn test_execute_asset_transfer() {
        let mut env = ExecutionEnvironment::new();
        env.add_balance("Alice", "ICN_TOKEN", 1000.0);

        let mut contract = SmartContract::new(
            ContractType::AssetTransfer,
            "Alice".to_string(),
            r#"{"from": "Alice", "to": "Bob", "asset": "ICN_TOKEN", "amount": 100.0}"#.to_string(),
        );
        
        contract.activate();
        
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.get_balance("Alice", "ICN_TOKEN"), 900.0);
        assert_eq!(env.get_balance("Bob", "ICN_TOKEN"), 100.0);
    }

    #[test]
    fn test_execute_proposal() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract::new(
            ContractType::Proposal,
            "Charlie".to_string(),
            r#"{"title": "New Project", "description": "Start a community garden", "options": ["Approve", "Reject"], "voting_period": 604800, "quorum": 0.5}"#.to_string(),
        );

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.proposals.len(), 1);
        assert!(env.proposals.contains_key(&contract.id));
    }

    #[test]
    fn test_execute_governance_vote() {
        let mut env = ExecutionEnvironment::new();
        let proposal_id = "proposal_1".to_string();
        let mut contract = SmartContract::new(
            ContractType::GovernanceVote,
            "Dave".to_string(),
            format!(r#"{{"proposal_id": "{}", "voter": "Dave", "vote": true}}"#, proposal_id),
        );

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.votes.len(), 1);
        assert_eq!(env.votes[&proposal_id], vec![("Dave".to_string(), true)]);
    }

    #[test]
    fn test_execute_service_agreement() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract::new(
            ContractType::ServiceAgreement,
            "Eve".to_string(),
            r#"{"provider": "Eve", "consumer": "Frank", "service": "Web Development", "terms": "Develop a website for 1000 ICN_TOKEN", "start_date": "2023-07-01T00:00:00Z", "end_date": "2023-08-01T00:00:00Z"}"#.to_string(),
        );

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.service_agreements.len(), 1);
        assert!(env.service_agreements.contains_key(&contract.id));
    }

    #[test]
    fn test_execute_resource_allocation() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract::new(
            ContractType::ResourceAllocation,
            "Grace".to_string(),
            r#"{"resource": "Computing Power", "amount": 100.0, "recipient": "Research Team", "duration": 2592000}"#.to_string(),
        );

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.resource_allocations.len(), 1);
        assert!(env.resource_allocations.contains_key(&contract.id));
    }

    #[test]
    fn test_execute_identity_verification() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract::new(
            ContractType::IdentityVerification,
            "Henry".to_string(),
            r#"{"user_id": "Henry", "verification_data": "Passport: AB123456", "verification_method": "Government ID", "expiration": "2025-07-01T00:00:00Z"}"#.to_string(),
        );

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.identities.len(), 1);
        assert!(env.identities.contains_key("Henry"));
    }

    #[test]
    fn test_execute_cooperative_membership() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract::new(
            ContractType::CooperativeMembership,
            "Ivy".to_string(),
            r#"{"user_id": "Ivy", "membership_type": "Full Member", "join_date": "2023-07-01T00:00:00Z", "subscription_period": 31536000}"#.to_string(),
        );

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.memberships.len(), 1);
        assert!(env.memberships.contains_key("Ivy"));
    }

    #[test]
    fn test_execute_custom_contract() {
        let mut env = ExecutionEnvironment::new();
        let mut contract = SmartContract::new(
            ContractType::Custom("DataSharing".to_string()),
            "Jack".to_string(),
            r#"{"data_provider": "Jack", "data_consumer": "Research Institute", "dataset": "Anonymous Health Records", "usage_terms": "Research purposes only", "compensation": 500}"#.to_string(),
        );

        contract.activate();
        assert!(contract.execute(&mut env).is_ok());
        assert_eq!(env.custom_contracts.len(), 1);
        assert!(env.custom_contracts.contains_key(&contract.id));
    }

    #[test]
    fn test_contract_lifecycle() {
        let mut contract = SmartContract::new(
            ContractType::AssetTransfer,
            "Alice".to_string(),
            r#"{"from": "Alice", "to": "Bob", "asset": "ICN_TOKEN", "amount": 100.0}"#.to_string(),
        );

        assert_eq!(contract.status, ContractStatus::Pending);

        contract.activate();
        assert_eq!(contract.status, ContractStatus::Active);

        contract.complete();
        assert_eq!(contract.status, ContractStatus::Completed);

        contract.terminate();
        assert_eq!(contract.status, ContractStatus::Terminated);
    }
}
