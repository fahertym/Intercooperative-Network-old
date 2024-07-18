use std::sync::{Arc, RwLock};
use std::error::Error;
use std::fmt;

pub mod blockchain;
pub mod consensus;
pub mod currency;
pub mod governance;
pub mod identity;
pub mod network;
pub mod node;
pub mod smart_contract;
pub mod vm;
pub mod sharding;
pub mod api;
pub mod error;

pub use blockchain::{Block, Transaction, Blockchain};
pub use currency::CurrencyType;
pub use governance::{DemocraticSystem, ProposalCategory, ProposalType};
pub use identity::DecentralizedIdentity;
pub use network::{Node, Network, Packet, PacketType};
pub use node::{ContentStore, ForwardingInformationBase, PendingInterestTable};
pub use smart_contract::{SmartContract, ExecutionEnvironment};
pub use vm::{CoopVM, Opcode};
pub use sharding::ShardingManager;

#[derive(Debug)]
pub struct CustomError(String);

impl Error for CustomError {}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct IcnNode {
    pub content_store: Arc<RwLock<ContentStore>>,
    pub pit: Arc<RwLock<PendingInterestTable>>,
    pub fib: Arc<RwLock<ForwardingInformationBase>>,
    pub blockchain: Arc<RwLock<Blockchain>>,
    pub coop_vm: Arc<RwLock<CoopVM>>,
    pub sharding_manager: Arc<RwLock<ShardingManager>>,
    pub execution_environment: Arc<RwLock<ExecutionEnvironment>>,
}

impl IcnNode {
    pub fn new() -> Self {
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let coop_vm = Arc::new(RwLock::new(CoopVM::new(Vec::new())));
        let sharding_manager = Arc::new(RwLock::new(ShardingManager::new(4, 10)));

        IcnNode {
            content_store: Arc::new(RwLock::new(ContentStore::new())),
            pit: Arc::new(RwLock::new(PendingInterestTable::new())),
            fib: Arc::new(RwLock::new(ForwardingInformationBase::new())),
            blockchain,
            coop_vm,
            sharding_manager,
            execution_environment: Arc::new(RwLock::new(ExecutionEnvironment::new())),
        }
    }

    pub fn process_cross_shard_transaction(&self, transaction: &Transaction) -> Result<(), Box<dyn Error>> {
        let mut sharding_manager = self.sharding_manager.write().unwrap();
        let from_shard = sharding_manager.get_shard_for_address(&transaction.from);
        let to_shard = sharding_manager.get_shard_for_address(&transaction.to);

        println!("Processing transaction from shard {} to shard {}", from_shard, to_shard);

        if from_shard != to_shard {
            sharding_manager.transfer_between_shards(from_shard, to_shard, transaction)
                .map_err(|e| Box::new(CustomError(e.to_string())) as Box<dyn Error>)
        } else {
            // Process transaction within the same shard
            sharding_manager.process_transaction(from_shard, transaction)
                .map_err(|e| Box::new(CustomError(e.to_string())) as Box<dyn Error>)
        }
    }

    pub fn execute_smart_contract(&self, contract: Box<dyn SmartContract>) -> Result<String, String> {
        let mut execution_environment = self.execution_environment.write().unwrap();
        contract.execute(&mut execution_environment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::CurrencyType;
    use rand::rngs::OsRng;
    use ed25519_dalek::Keypair;

    #[test]
    fn test_cross_shard_transaction() {
        let node = IcnNode::new();

        // Initialize balances
        {
            let mut sharding_manager = node.sharding_manager.write().unwrap();
            sharding_manager.add_address_to_shard("Alice".to_string(), 0);
            sharding_manager.add_address_to_shard("Bob".to_string(), 1);
            sharding_manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0).unwrap();
        }

        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            500.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair).unwrap();

        assert!(node.process_cross_shard_transaction(&transaction).is_ok());

        // Check balances after transaction
        let sharding_manager = node.sharding_manager.read().unwrap();
        assert_eq!(sharding_manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds).unwrap(), 500.0);
        assert_eq!(sharding_manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds).unwrap(), 500.0);
    }
}