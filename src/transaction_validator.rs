use crate::blockchain::{Transaction, Blockchain};

pub struct TransactionValidator;

impl TransactionValidator {
    pub fn validate_transaction(transaction: &Transaction, blockchain: &Blockchain) -> bool {
        if !Self::is_double_spend(transaction, blockchain) &&
           Self::validate_currency_and_amount(transaction) &&
           Self::check_sufficient_balance(transaction, blockchain) {
            true
        } else {
            false
        }
    }

    fn is_double_spend(_transaction: &Transaction, _blockchain: &Blockchain) -> bool {
        // TODO: Implement double spend check
        false
    }

    fn validate_currency_and_amount(transaction: &Transaction) -> bool {
        transaction.amount > 0.0
    }

    fn check_sufficient_balance(_transaction: &Transaction, _blockchain: &Blockchain) -> bool {
        // TODO: Implement balance check
        true
    }
}