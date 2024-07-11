// Filename: smart_contract.rs

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

// ================================================
// Enums and Structs for Smart Contracts
// ================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContract {
    pub id: String,
    pub contract_type: ContractType,
    pub creator: String,
    pub created_at: DateTime<Utc>,
    pub content: String,
    pub status: ContractStatus,
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContractStatus {
    Pending,
    Active,
    Completed,
    Terminated,
}

// ================================================
// Implementation of SmartContract
// ================================================

impl SmartContract {
    // Constructor for creating a new SmartContract
    pub fn new(contract_type: ContractType, creator: String, content: String) -> Self {
        SmartContract {
            id: format!("contract_{}", Utc::now().timestamp()),
            contract_type,
            creator,
            created_at: Utc::now(),
            content,
            status: ContractStatus::Pending,
        }
    }

    // Methods to change the status of the contract
    pub fn activate(&mut self) {
        self.status = ContractStatus::Active;
    }

    pub fn complete(&mut self) {
        self.status = ContractStatus::Completed;
    }

    pub fn terminate(&mut self) {
        self.status = ContractStatus::Terminated;
    }

    // Main execute function that dispatches to specific execution functions
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

    fn execute_proposal(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: ProposalParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse proposal params: {}", e))?;

        env.proposals.insert(self.id.clone(), params);
        Ok(())
    }

    fn execute_service_agreement(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: ServiceAgreementParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse service agreement params: {}", e))?;

        env.service_agreements.insert(self.id.clone(), params);
        Ok(())
    }

    fn execute_governance_vote(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: GovernanceVoteParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse governance vote params: {}", e))?;

        env.votes.entry(params.proposal_id)
            .or_insert_with(Vec::new)
            .push((params.voter, params.vote));
        Ok(())
    }

    fn execute_resource_allocation(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: ResourceAllocationParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse resource allocation params: {}", e))?;

        // Add debug statement
        println!("Parsed ResourceAllocationParams: {:?}", params);

        env.resource_allocations.insert(self.id.clone(), params);
        Ok(())
    }

    fn execute_identity_verification(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: IdentityVerificationParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse identity verification params: {}", e))?;

        env.identities.insert(params.user_id.clone(), params);
        Ok(())
    }

    fn execute_cooperative_membership(&self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let params: CooperativeMembershipParams = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse cooperative membership params: {}", e))?;

        // Add debug statement
        println!("Parsed CooperativeMembershipParams: {:?}", params);

        env.memberships.insert(params.user_id.clone(), params);
        Ok(())
    }

    fn execute_custom_contract(&self, env: &mut ExecutionEnvironment, name: &str) -> Result<(), String> {
        env.custom_contracts.insert(self.id.clone(), (name.to_string(), self.content.clone()));
        Ok(())
    }
}

// ================================================
// Structs for Execution Environment and Params
// ================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEnvironment {
    pub balances: HashMap<String, HashMap<String, f64>>,
    pub proposals: HashMap<String, ProposalParams>,
    pub votes: HashMap<String, Vec<(String, bool)>>,
    pub service_agreements: HashMap<String, ServiceAgreementParams>,
    pub resource_allocations: HashMap<String, ResourceAllocationParams>,
    pub identities: HashMap<String, IdentityVerificationParams>,
    pub memberships: HashMap<String, CooperativeMembershipParams>,
    pub custom_contracts: HashMap<String, (String, String)>,
}

// Implementation for the Execution Environment
impl ExecutionEnvironment {
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

    pub fn add_balance(&mut self, user: &str, asset: &str, amount: f64) {
        self.balances
            .entry(user.to_string())
            .or_insert_with(HashMap::new)
            .entry(asset.to_string())
            .and_modify(|balance| *balance += amount)
            .or_insert(amount);
    }

    pub fn get_balance(&self, user: &str, asset: &str) -> f64 {
        self.balances
            .get(user)
            .and_then(|assets| assets.get(asset))
            .cloned()
            .unwrap_or(0.0)
    }

    pub fn tally_votes(&self, proposal_id: &str) -> (usize, usize) {
        let votes = self.votes.get(proposal_id).cloned().unwrap_or_default();
        let (approve, reject): (Vec<_>, Vec<_>) = votes.into_iter().partition(|(_, vote)| *vote);
        (approve.len(), reject.len())
    }
}

// ================================================
// Structs for Specific Params
// ================================================

#[derive(Debug, Serialize, Deserialize)]
struct AssetTransferParams {
    from: String,
    to: String,
    asset: String,
    amount: f64,
}

// Making ProposalParams public to avoid visibility issues
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

// ================================================
// Utility Functions and Modules
// ================================================

pub fn parse_contract(input: &str) -> Result<SmartContract, String> {
    let contract: SmartContract = serde_json::from_str(input)
        .map_err(|e| format!("Failed to parse contract: {}", e))?;
    Ok(contract)
}

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

// ================================================
// Unit Tests
// ================================================

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
