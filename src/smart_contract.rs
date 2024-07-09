use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ContractType {
    AssetTransfer,
    Proposal,
    ServiceAgreement,
    GovernanceVote,
    ResourceAllocation,
    IdentityVerification,
    CooperativeMembership,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SmartContract {
    pub id: String,
    pub contract_type: ContractType,
    pub creator: String,
    pub created_at: DateTime<Utc>,
    pub content: String,
    pub state: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetTransferContract {
    pub from: String,
    pub to: String,
    pub asset: String,
    pub amount: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProposalContract {
    pub title: String,
    pub description: String,
    pub voting_period: Duration,
    pub options: Vec<String>,
    pub quorum: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceAgreementContract {
    pub service_provider: String,
    pub client: String,
    pub service_description: String,
    pub payment_amount: f64,
    pub deadline: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GovernanceVoteContract {
    pub proposal_id: String,
    pub voter: String,
    pub vote: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResourceAllocationContract {
    pub resource_type: String,
    pub amount: f64,
    pub recipient: String,
    pub duration: Duration,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IdentityVerificationContract {
    pub subject: String,
    pub verifier: String,
    pub credentials: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CooperativeMembershipContract {
    pub cooperative_id: String,
    pub member_id: String,
    pub membership_type: String,
    pub join_date: DateTime<Utc>,
}

pub struct ExecutionEnvironment {
    pub balances: HashMap<String, HashMap<String, f64>>,
    pub votes: HashMap<String, HashMap<String, i32>>,
    pub identities: HashMap<String, HashMap<String, String>>,
    pub cooperatives: HashMap<String, Vec<String>>,
    pub resources: HashMap<String, HashMap<String, f64>>,
}

impl SmartContract {
    pub fn new(contract_type: ContractType, creator: String, content: String) -> Self {
        SmartContract {
            id: format!("contract_{}", Utc::now().timestamp()),
            contract_type,
            creator,
            created_at: Utc::now(),
            content,
            state: HashMap::new(),
        }
    }

    pub fn execute(&mut self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        match self.contract_type {
            ContractType::AssetTransfer => self.execute_asset_transfer(env),
            ContractType::Proposal => self.execute_proposal(env),
            ContractType::ServiceAgreement => self.execute_service_agreement(env),
            ContractType::GovernanceVote => self.execute_governance_vote(env),
            ContractType::ResourceAllocation => self.execute_resource_allocation(env),
            ContractType::IdentityVerification => self.execute_identity_verification(env),
            ContractType::CooperativeMembership => self.execute_cooperative_membership(env),
        }
    }

    fn execute_asset_transfer(&mut self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let contract: AssetTransferContract = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse AssetTransferContract: {}", e))?;

        let from_balance = env.balances.entry(contract.from.clone()).or_default();
        let balance = from_balance.entry(contract.asset.clone()).or_default();

        if *balance < contract.amount {
            return Err("Insufficient funds".to_string());
        }

        *balance -= contract.amount;

        let to_balance = env.balances.entry(contract.to.clone()).or_default();
        *to_balance.entry(contract.asset.clone()).or_default() += contract.amount;

        Ok(())
    }

    fn execute_proposal(&mut self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let contract: ProposalContract = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse ProposalContract: {}", e))?;

        let voting_end = self.created_at + contract.voting_period;
        if Utc::now() < voting_end {
            return Err("Voting period has not ended yet".to_string());
        }

        let votes = env.votes.get(&self.id).ok_or("No votes found")?;
        let total_votes: i32 = votes.values().sum();
        
        if (total_votes as f64) < contract.quorum {
            return Err("Quorum not reached".to_string());
        }

        let mut max_votes = 0;
        let mut winning_option = String::new();

        for (option, vote_count) in votes {
            if *vote_count > max_votes {
                max_votes = *vote_count;
                winning_option = option.clone();
            }
        }

        self.state.insert("result".to_string(), winning_option);
        Ok(())
    }

    fn execute_service_agreement(&mut self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let contract: ServiceAgreementContract = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse ServiceAgreementContract: {}", e))?;

        if Utc::now() > contract.deadline {
            return Err("Service agreement deadline has passed".to_string());
        }

        let client_balance = env.balances.entry(contract.client.clone()).or_default();
        let balance = client_balance.entry("ICN_TOKEN".to_string()).or_default();

        if *balance < contract.payment_amount {
            return Err("Insufficient funds for payment".to_string());
        }

        *balance -= contract.payment_amount;

        let provider_balance = env.balances.entry(contract.service_provider.clone()).or_default();
        *provider_balance.entry("ICN_TOKEN".to_string()).or_default() += contract.payment_amount;

        self.state.insert("status".to_string(), "completed".to_string());
        Ok(())
    }

    fn execute_governance_vote(&mut self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let contract: GovernanceVoteContract = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse GovernanceVoteContract: {}", e))?;

        let votes = env.votes.entry(contract.proposal_id.clone()).or_default();
        *votes.entry(contract.vote.clone()).or_default() += 1;

        Ok(())
    }

    fn execute_resource_allocation(&mut self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let contract: ResourceAllocationContract = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse ResourceAllocationContract: {}", e))?;

        let resources = env.resources.entry(contract.resource_type.clone()).or_default();
        let available = resources.entry("available".to_string()).or_default();

        if *available < contract.amount {
            return Err("Insufficient resources".to_string());
        }

        *available -= contract.amount;
        *resources.entry(contract.recipient.clone()).or_default() += contract.amount;

        self.state.insert("expiration".to_string(), (Utc::now() + contract.duration).to_rfc3339());
        Ok(())
    }

    fn execute_identity_verification(&mut self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let contract: IdentityVerificationContract = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse IdentityVerificationContract: {}", e))?;

        let identity = env.identities.entry(contract.subject.clone()).or_default();
        for (key, value) in contract.credentials {
            identity.insert(key, value);
        }

        self.state.insert("verified_by".to_string(), contract.verifier);
        Ok(())
    }

    fn execute_cooperative_membership(&mut self, env: &mut ExecutionEnvironment) -> Result<(), String> {
        let contract: CooperativeMembershipContract = serde_json::from_str(&self.content)
            .map_err(|e| format!("Failed to parse CooperativeMembershipContract: {}", e))?;

        let members = env.cooperatives.entry(contract.cooperative_id.clone()).or_default();
        if members.contains(&contract.member_id) {
            return Err("Member already exists in the cooperative".to_string());
        }

        members.push(contract.member_id.clone());

        self.state.insert("status".to_string(), "active".to_string());
        self.state.insert("join_date".to_string(), contract.join_date.to_rfc3339());
        Ok(())
    }
}

pub fn parse_contract(input: &str) -> Result<SmartContract, String> {
    let lines: Vec<&str> = input.lines().collect();
    if lines.len() < 2 {
        return Err("Invalid contract format".to_string());
    }

    let (contract_type, content) = match lines[0].trim() {
        "Asset Transfer" => {
            let contract = AssetTransferContract {
                from: lines.get(2).ok_or("Missing 'from' field")?.trim().to_string(),
                to: lines.get(3).ok_or("Missing 'to' field")?.trim().to_string(),
                asset: lines.get(4).ok_or("Missing 'asset' field")?.trim().to_string(),
                amount: lines.get(5).ok_or("Missing 'amount' field")?
                    .trim().parse().map_err(|_| "Invalid amount")?,
            };
            (ContractType::AssetTransfer, serde_json::to_string(&contract).unwrap())
        },
        "Proposal" => {
            let contract = ProposalContract {
                title: lines.get(2).ok_or("Missing 'title' field")?.trim().to_string(),
                description: lines.get(3).ok_or("Missing 'description' field")?.trim().to_string(),
                voting_period: Duration::seconds(lines.get(4).ok_or("Missing 'voting period' field")?
                    .trim().parse().map_err(|_| "Invalid voting period")?),
                options: lines[5..].iter().take_while(|&&s| !s.starts_with("Quorum:")).map(|s| s.trim().to_string()).collect(),
                quorum: lines.iter().find(|&&s| s.starts_with("Quorum:"))
                    .ok_or("Missing 'Quorum' field")?
                    .trim_start_matches("Quorum:")
                    .trim().parse().map_err(|_| "Invalid quorum")?,
            };
            (ContractType::Proposal, serde_json::to_string(&contract).unwrap())
        },
        "Service Agreement" => {
            let contract = ServiceAgreementContract {
                service_provider: lines.get(2).ok_or("Missing 'service provider' field")?.trim().to_string(),
                client: lines.get(3).ok_or("Missing 'client' field")?.trim().to_string(),
                service_description: lines.get(4).ok_or("Missing 'service description' field")?.trim().to_string(),
                payment_amount: lines.get(5).ok_or("Missing 'payment amount' field")?
                    .trim().parse().map_err(|_| "Invalid payment amount")?,
                deadline: DateTime::from_str(lines.get(6).ok_or("Missing 'deadline' field")?.trim())
                    .map_err(|_| "Invalid deadline")?,
            };
            (ContractType::ServiceAgreement, serde_json::to_string(&contract).unwrap())
        },
        "Governance Vote" => {
            let contract = GovernanceVoteContract {
                proposal_id: lines.get(2).ok_or("Missing 'proposal id' field")?.trim().to_string(),
                voter: lines.get(3).ok_or("Missing 'voter' field")?.trim().to_string(),
                vote: lines.get(4).ok_or("Missing 'vote' field")?.trim().to_string(),
            };
            (ContractType::GovernanceVote, serde_json::to_string(&contract).unwrap())
        },
        "Resource Allocation" => {
            let contract = ResourceAllocationContract {
                resource_type: lines.get(2).ok_or("Missing 'resource type' field")?.trim().to_string(),
                amount: lines.get(3).ok_or("Missing 'amount' field")?
                    .trim().parse().map_err(|_| "Invalid amount")?,
                recipient: lines.get(4).ok_or("Missing 'recipient' field")?.trim().to_string(),
                duration: Duration::seconds(lines.get(5).ok_or("Missing 'duration' field")?
                    .trim().parse().map_err(|_| "Invalid duration")?),
            };
            (ContractType::ResourceAllocation, serde_json::to_string(&contract).unwrap())
        },
        "Identity Verification" => {
            let mut credentials = HashMap::new();
            for line in lines.iter().skip(4) {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    credentials.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                }
            }
            let contract = IdentityVerificationContract {
                subject: lines.get(2).ok_or("Missing 'subject' field")?.trim().to_string(),
                verifier: lines.get(3).ok_or("Missing 'verifier' field")?.trim().to_string(),
                credentials,
            };
            (ContractType::IdentityVerification, serde_json::to_string(&contract).unwrap())
        },
        "Cooperative Membership" => {
            let contract = CooperativeMembershipContract {
                cooperative_id: lines.get(2).ok_or("Missing 'cooperative id' field")?.trim().to_string(),
                member_id: lines.get(3).ok_or("Missing 'member id' field")?.trim().to_string(),
                membership_type: lines.get(4).ok_or("Missing 'membership type' field")?.trim().to_string(),
                join_date: DateTime::from_str(lines.get(5).ok_or("Missing 'join date' field")?.trim())
                    .map_err(|_| "Invalid join date")?,
            };
            (ContractType::CooperativeMembership, serde_json::to_string(&contract).unwrap())
        },
        _ => return Err("Unknown contract type".to_string()),
    };

    Ok(SmartContract::new(contract_type, lines[1].trim().to_string(), content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_asset_transfer_contract() {
        let input = "Asset Transfer
Creator: Alice
From: Alice
To: Bob
Asset: ICN_TOKEN
Amount: 100.0";
        let contract = parse_contract(input).unwrap();
        assert_eq!(contract.contract_type, ContractType::AssetTransfer);
        assert_eq!(contract.creator, "Alice");
    }

    #[test]
    fn test_execute_asset_transfer() {
        let mut contract = parse_contract("Asset Transfer
Creator: Alice
From: Alice
To: Bob
Asset: ICN_TOKEN
Amount: 100.0").unwrap();

        let mut env = ExecutionEnvironment {
            balances: HashMap::new(),
            votes: HashMap::new(),
            identities: HashMap::new(),
            cooperatives: HashMap::new(),
            resources: HashMap::new(),
        };

        env.balances.insert("Alice".to_string(), HashMap::new());
        env.balances.get_mut("Alice").unwrap().insert("ICN_TOKEN".to_string(), 200.0);

        contract.execute(&mut env).unwrap();

        assert_eq!(env.balances.get("Alice").unwrap().get("ICN_TOKEN").unwrap(), &100.0);
        assert_eq!(env.balances.get("Bob").unwrap().get("ICN_TOKEN").unwrap(), &100.0);
    }

    #[test]
    fn test_parse_proposal_contract() {
        let input = "Proposal
Creator: Alice
Title: New Community Project
Description: Implement a recycling program
Voting Period: 604800
Option 1: Approve
Option 2: Reject
Quorum: 0.5";
        let contract = parse_contract(input).unwrap();
        assert_eq!(contract.contract_type, ContractType::Proposal);
        assert_eq!(contract.creator, "Alice");
    }

    #[test]
    fn test_execute_proposal() {
        let mut contract = parse_contract("Proposal
Creator: Alice
Title: New Community Project
Description: Implement a recycling program
Voting Period: 604800
Option 1: Approve
Option 2: Reject
Quorum: 0.5").unwrap();

        let mut env = ExecutionEnvironment {
            balances: HashMap::new(),
            votes: HashMap::new(),
            identities: HashMap::new(),
            cooperatives: HashMap::new(),
            resources: HashMap::new(),
        };

        let mut votes = HashMap::new();
        votes.insert("Approve".to_string(), 10);
        votes.insert("Reject".to_string(), 5);
        env.votes.insert(contract.id.clone(), votes);

        // Set the created_at time to the past so that the voting period has ended
        contract.created_at = Utc::now() - Duration::weeks(2);

        contract.execute(&mut env).unwrap();

        assert_eq!(contract.state.get("result").unwrap(), "Approve");
    }
}