use serde::{Serialize, Deserialize, Serializer, Deserializer};
use chrono::{DateTime, Utc, Duration};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    #[serde(with = "chrono::serde::ts_seconds")]
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
    #[serde(with = "duration_seconds")]
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
    #[serde(with = "chrono::serde::ts_seconds")]
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
    #[serde(with = "duration_seconds")]
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
    #[serde(with = "chrono::serde::ts_seconds")]
    pub join_date: DateTime<Utc>,
}

#[derive(Debug, Default, Clone)]
pub struct ExecutionEnvironment {
    pub balances: HashMap<String, HashMap<String, f64>>,
    pub votes: HashMap<String, HashMap<String, i32>>,
    pub identities: HashMap<String, HashMap<String, String>>,
    pub cooperatives: HashMap<String, Vec<String>>,
    pub resources: HashMap<String, HashMap<String, f64>>,
}

impl ExecutionEnvironment {
    pub fn new() -> Self {
        ExecutionEnvironment {
            balances: HashMap::new(),
            votes: HashMap::new(),
            identities: HashMap::new(),
            cooperatives: HashMap::new(),
            resources: HashMap::new(),
        }
    }
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
        members.push(contract.member_id.clone());
        self.state.insert("membership_type".to_string(), contract.membership_type);
        Ok(())
    }
}

impl SmartContract {
    pub fn create_asset_transfer(from: String, to: String, asset: String, amount: f64) -> Self {
        let contract = AssetTransferContract { from, to, asset, amount };
        SmartContract::new(
            ContractType::AssetTransfer,
            "Creator".to_string(),
            serde_json::to_string(&contract).unwrap(),
        )
    }

    pub fn create_proposal(title: String, description: String, voting_period: Duration, options: Vec<String>, quorum: f64) -> Self {
        let contract = ProposalContract {
            title,
            description,
            voting_period,
            options,
            quorum,
        };
        SmartContract::new(
            ContractType::Proposal,
            "Creator".to_string(),
            serde_json::to_string(&contract).unwrap(),
        )
    }

    pub fn create_service_agreement(service_provider: String, client: String, service_description: String, payment_amount: f64, deadline: DateTime<Utc>) -> Self {
        let contract = ServiceAgreementContract {
            service_provider,
            client,
            service_description,
            payment_amount,
            deadline,
        };
        SmartContract::new(
            ContractType::ServiceAgreement,
            "Creator".to_string(),
            serde_json::to_string(&contract).unwrap(),
        )
    }

    pub fn create_governance_vote(proposal_id: String, voter: String, vote: String) -> Self {
        let contract = GovernanceVoteContract {
            proposal_id,
            voter,
            vote,
        };
        SmartContract::new(
            ContractType::GovernanceVote,
            "Creator".to_string(),
            serde_json::to_string(&contract).unwrap(),
        )
    }

    pub fn create_resource_allocation(resource_type: String, amount: f64, recipient: String, duration: Duration) -> Self {
        let contract = ResourceAllocationContract {
            resource_type,
            amount,
            recipient,
            duration,
        };
        SmartContract::new(
            ContractType::ResourceAllocation,
            "Creator".to_string(),
            serde_json::to_string(&contract).unwrap(),
        )
    }

    pub fn create_identity_verification(subject: String, verifier: String, credentials: HashMap<String, String>) -> Self {
        let contract = IdentityVerificationContract {
            subject,
            verifier,
            credentials,
        };
        SmartContract::new(
            ContractType::IdentityVerification,
            "Creator".to_string(),
            serde_json::to_string(&contract).unwrap(),
        )
    }

    pub fn create_cooperative_membership(cooperative_id: String, member_id: String, membership_type: String, join_date: DateTime<Utc>) -> Self {
        let contract = CooperativeMembershipContract {
            cooperative_id,
            member_id,
            membership_type,
            join_date,
        };
        SmartContract::new(
            ContractType::CooperativeMembership,
            "Creator".to_string(),
            serde_json::to_string(&contract).unwrap(),
        )
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
                deadline: DateTime::parse_from_rfc3339(lines.get(6).ok_or("Missing 'deadline' field")?.trim())
                    .map_err(|_| "Invalid deadline")?.with_timezone(&Utc),
            };
            (ContractType::ServiceAgreement, serde_json::to_string(&contract).unwrap())
        },
        _ => return Err("Unknown contract type".to_string()),
    };

    Ok(SmartContract::new(contract_type, "Creator".to_string(), content))
}

mod duration_seconds {
    use serde::{self, Deserialize, Serializer, Deserializer};
    use chrono::Duration;

    pub fn serialize<S>(dur: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(dur.num_seconds())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = i64::deserialize(deserializer)?;
        Ok(Duration::seconds(secs))
    }
}
