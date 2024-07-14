use crate::error::{BlockchainError, BlockchainResult};
use log::{debug, info};
use rand::rngs::OsRng;
use rand::RngCore;
use std::sync::{Arc, Mutex};
use crate::blockchain::{Block, Transaction};
use crate::smart_contract::{SmartContract, ExecutionEnvironment};
use crate::consensus::Consensus;
use crate::sharding::ShardingManagerTrait;
use std::collections::HashMap;

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub smart_contracts: HashMap<String, SmartContract>,
    pub execution_environment: ExecutionEnvironment,
    pub consensus: Arc<Mutex<Consensus>>,
    pub sharding_manager: Arc<Mutex<dyn ShardingManagerTrait + Send + 'static>>,
}

impl Blockchain {
    pub fn new(consensus: Arc<Mutex<Consensus>>, sharding_manager: Arc<Mutex<dyn ShardingManagerTrait + Send + 'static>>) -> Self {
        let mut blockchain = Blockchain {
            chain: vec![],
            pending_transactions: vec![],
            smart_contracts: HashMap::new(),
            execution_environment: ExecutionEnvironment::new(),
            consensus,
            sharding_manager,
        };

        let genesis_block = Block::new(0, vec![], String::new());
        blockchain.chain.push(genesis_block);

        info!("Blockchain initialized with genesis block");

        blockchain
    }

    pub fn add_transaction(&mut self, transaction: Transaction) -> BlockchainResult<()> {
        let sharding_manager = self.sharding_manager.lock().map_err(|_| BlockchainError::MutexLockError)?;
        let from_shard = sharding_manager.get_shard_for_address(&transaction.from);
        let to_shard = sharding_manager.get_shard_for_address(&transaction.to);
        drop(sharding_manager);

        if from_shard == to_shard {
            debug!("Adding transaction to pending transactions: {:?}", transaction);
            self.pending_transactions.push(transaction);
            Ok(())
        } else {
            info!("Processing cross-shard transaction: {:?}", transaction);
            self.pending_transactions.push(transaction.clone());
            self.process_cross_shard_transaction(transaction)
        }
    }

    pub fn create_block(&mut self) -> BlockchainResult<()> {
        let previous_block = self.chain.last().ok_or(BlockchainError::EmptyBlockchain)?;
        let new_block = Block::new(self.chain.len() as u64, self.pending_transactions.clone(), previous_block.hash.clone());

        self.validate_block(&new_block)?;

        info!("Creating new block: {:?}", new_block);
        self.chain.push(new_block);
        self.pending_transactions.clear();

        Ok(())
    }

    pub fn validate_block(&self, block: &Block) -> BlockchainResult<()> {
        if let Some(previous_block) = self.chain.last() {
            if block.previous_hash != previous_block.hash {
                return Err(BlockchainError::InvalidBlock("Invalid previous hash".to_string()));
            }
        }

        for transaction in &block.transactions {
            self.validate_transaction(transaction)?;
        }

        if block.hash != block.calculate_hash() {
            return Err(BlockchainError::InvalidBlock("Invalid block hash".to_string()));
        }

        Ok(())
    }

    pub fn validate_transaction(&self, transaction: &Transaction) -> BlockchainResult<()> {
        let sharding_manager = self.sharding_manager.lock().map_err(|_| BlockchainError::MutexLockError)?;
        let balance = sharding_manager.get_balance(&transaction.from, &transaction.currency_type);
        if balance < transaction.amount {
            return Err(BlockchainError::InvalidTransaction("Insufficient balance".to_string()));
        }

        if !transaction.verify().map_err(|e| BlockchainError::InvalidTransaction(e.to_string()))? {
            return Err(BlockchainError::InvalidTransaction("Invalid transaction signature".to_string()));
        }

        Ok(())
    }

    pub fn get_latest_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn is_chain_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            if current_block.hash != current_block.calculate_hash() {
                return false;
            }

            if current_block.previous_hash != previous_block.hash {
                return false;
            }
        }
        true
    }

    pub fn store_smart_contract(&mut self, contract: SmartContract) -> BlockchainResult<()> {
        self.smart_contracts.insert(contract.id.clone(), contract);
        Ok(())
    }

    pub fn get_smart_contract(&self, id: &str) -> Option<&SmartContract> {
        self.smart_contracts.get(id)
    }

    pub fn update_smart_contract(&mut self, id: &str, updated_contract: SmartContract) -> BlockchainResult<()> {
        self.smart_contracts.insert(id.to_string(), updated_contract);
        Ok(())
    }

    pub fn remove_smart_contract(&mut self, id: &str) -> BlockchainResult<()> {
        self.smart_contracts.remove(id);
        Ok(())
    }

    pub fn deploy_smart_contract(&mut self, contract: SmartContract) -> BlockchainResult<()> {
        if self.smart_contracts.contains_key(&contract.id) {
            return Err(BlockchainError::SmartContractError("Smart contract with this ID already exists".to_string()));
        }
        self.smart_contracts.insert(contract.id.clone(), contract);
        Ok(())
    }

    pub fn execute_smart_contracts(&mut self) -> BlockchainResult<()> {
        let block = self.chain.last_mut().ok_or(BlockchainError::EmptyBlockchain)?;
        let transactions = block.transactions.clone();
        for transaction in transactions {
            if let Some(ref contract) = transaction.smart_contract {
                let result = contract.execute(&mut self.execution_environment)
                    .map_err(|e| BlockchainError::SmartContractError(e))?;
                block.add_smart_contract_result(contract.id.clone(), result, transaction.gas_limit);
            }
        }
        Ok(())
    }

    pub fn select_proposer(&self) -> Option<String> {
        let consensus = self.consensus.lock().unwrap();
        let total_reputation: f64 = consensus.members.values().map(|member| member.reputation).sum();
        let mut rng = OsRng;
        let selection_point: f64 = rng.next_u32() as f64 / std::u32::MAX as f64 * total_reputation;

        let mut cumulative_reputation = 0.0;
        for member in consensus.members.values() {
            cumulative_reputation += member.reputation;
            if cumulative_reputation >= selection_point {
                return Some(member.id.clone());
            }
        }

        None
    }

    pub fn process_transactions(&mut self) -> BlockchainResult<()> {
        let transactions_to_process = self.pending_transactions.clone();
        self.pending_transactions.clear();

        for transaction in transactions_to_process {
            let sharding_manager = self.sharding_manager.lock().map_err(|_| BlockchainError::MutexLockError)?;
            let from_shard = sharding_manager.get_shard_for_address(&transaction.from);
            let to_shard = sharding_manager.get_shard_for_address(&transaction.to);
            drop(sharding_manager);

            if from_shard == to_shard {
                self.execute_transaction(&transaction)?;
            } else {
                self.process_cross_shard_transaction(transaction)?;
            }
        }

        Ok(())
    }

    pub fn process_cross_shard_transaction(&mut self, transaction: Transaction) -> BlockchainResult<()> {
        let mut sharding_manager = self.sharding_manager.lock().map_err(|_| BlockchainError::MutexLockError)?;

        let from_shard = sharding_manager.get_shard_for_address(&transaction.from);
        let to_shard = sharding_manager.get_shard_for_address(&transaction.to);

        sharding_manager.lock_funds(&transaction.from, &transaction.currency_type, transaction.amount, from_shard)
            .map_err(|e| BlockchainError::ShardingError(e))?;
        sharding_manager.create_prepare_block(&transaction, to_shard)
            .map_err(|e| BlockchainError::ShardingError(e))?;
        sharding_manager.commit_transaction(&transaction, to_shard)
            .map_err(|e| BlockchainError::ShardingError(e))?;

        Ok(())
    }

    fn execute_transaction(&mut self, transaction: &Transaction) -> BlockchainResult<()> {
        let mut sharding_manager = self.sharding_manager.lock().map_err(|_| BlockchainError::MutexLockError)?;
        let shard_id = sharding_manager.get_shard_for_address(&transaction.from);

        sharding_manager.lock_funds(&transaction.from, &transaction.currency_type, transaction.amount, shard_id)
            .map_err(|e| BlockchainError::ShardingError(e))?;
        sharding_manager.commit_transaction(transaction, shard_id)
            .map_err(|e| BlockchainError::ShardingError(e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::currency::CurrencyType;
    use rand::rngs::OsRng;
    use ed25519_dalek::Keypair;

    struct MockShardingManager;

    impl ShardingManagerTrait for MockShardingManager {
        fn get_shard_for_address(&self, address: &str) -> u64 {
            if address == "Alice" { 0 } else { 1 }
        }
        fn lock_funds(&mut self, _from: &str, _currency_type: &CurrencyType, _amount: f64, _shard_id: u64) -> Result<(), String> { Ok(()) }
        fn create_prepare_block(&mut self, _transaction: &Transaction, _shard_id: u64) -> Result<(), String> { Ok(()) }
        fn commit_transaction(&mut self, _transaction: &Transaction, _shard_id: u64) -> Result<(), String> { Ok(()) }
        fn get_balance(&self, address: &str, _currency_type: &CurrencyType) -> f64 {
            if address == "Alice" { 1000.0 } else { 0.0 }
        }
    }

    #[test]
    fn test_blockchain_creation() {
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let sharding_manager = Arc::new(Mutex::new(MockShardingManager));
        let blockchain = Blockchain::new(consensus, sharding_manager);
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.chain[0].index, 0);
    }

    #[test]
    fn test_add_transaction() {
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let sharding_manager = Arc::new(Mutex::new(MockShardingManager));
        let mut blockchain = Blockchain::new(consensus, sharding_manager);

        let mut csprng = OsRng;
        let keypair: Keypair = Keypair::generate(&mut csprng);
        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair).unwrap();

        assert!(blockchain.add_transaction(transaction).is_ok());
        assert_eq!(blockchain.pending_transactions.len(), 1);
    }

    #[test]
    fn test_create_block() {
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let sharding_manager = Arc::new(Mutex::new(MockShardingManager));
        let mut blockchain = Blockchain::new(consensus, sharding_manager);

        let mut csprng = OsRng;
        let keypair: Keypair = Keypair::generate(&mut csprng);
        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair).unwrap();

        blockchain.add_transaction(transaction).unwrap();
        assert!(blockchain.create_block().is_ok());
        assert_eq!(blockchain.chain.len(), 2);
    }

    #[test]
    fn test_blockchain_validity() {
        let consensus = Arc::new(Mutex::new(Consensus::new()));
        let sharding_manager = Arc::new(Mutex::new(MockShardingManager));
        let mut blockchain = Blockchain::new(consensus, sharding_manager);

        let mut csprng = OsRng;
        let keypair: Keypair = Keypair::generate(&mut csprng);
        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair).unwrap();

        blockchain.add_transaction(transaction).unwrap();
        blockchain.create_block().unwrap();

        assert!(blockchain.is_chain_valid());
    }
}