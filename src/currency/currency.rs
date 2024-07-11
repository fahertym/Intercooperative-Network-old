// Filename: src/currency.rs

// ===============================================
// Currency System Implementation
// ===============================================
// This file implements a comprehensive multi-currency system for our blockchain.
// It defines various types of currencies, their properties, and mechanisms for
// managing them. The system supports a diverse economy with different types of
// value representation, from basic needs to luxury goods and custom currencies.

// ===============================================
// Imports
// ===============================================
// These imports provide necessary functionality for our currency system:
// - chrono: For handling dates and times
// - std::collections::HashMap: For storing key-value pairs efficiently
// - serde: For serializing and deserializing data structures
// - std::fmt: For custom formatting of our types
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt;

// ===============================================
// CurrencyType Enum
// ===============================================
// This enum represents the different types of currencies in our system.
// Each variant corresponds to a specific economic sector or purpose.

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum CurrencyType {
    BasicNeeds,     // For essential goods and services (e.g., food, water)
    Education,      // For educational services and resources
    Environmental,  // For environmental initiatives and projects
    Community,      // For community projects and services
    Volunteer,      // For volunteer services and recognition
    Storage,        // For data storage services
    Processing,     // For computational power and services
    Energy,         // For energy resources and trading
    Luxury,         // For non-essential, high-end goods and services
    Service,        // For general services not covered by other categories
    Custom(String), // For user-defined currencies with custom names
}

// Implementation of the Display trait for CurrencyType
// This allows us to easily convert CurrencyType to a string representation
impl fmt::Display for CurrencyType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CurrencyType::Custom(name) => write!(f, "Custom({})", name),
            _ => write!(f, "{:?}", self),
        }
    }
}

// ===============================================
// Currency Struct
// ===============================================
// This struct represents a single currency in our system, containing
// all relevant information about its supply, creation, and issuance.

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Currency {
    pub currency_type: CurrencyType, // The type of this currency
    pub total_supply: f64,           // The total amount of this currency in circulation
    pub creation_date: DateTime<Utc>, // When this currency was first created
    pub last_issuance: DateTime<Utc>, // When new units were last issued
    pub issuance_rate: f64,           // The rate at which new units are issued
}

// Implementation of methods for the Currency struct
impl Currency {
    // Create a new Currency instance with the given parameters
    pub fn new(currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) -> Self {
        let now = Utc::now();
        Currency {
            currency_type,
            total_supply: initial_supply,
            creation_date: now,
            last_issuance: now,
            issuance_rate,
        }
    }

    // Mint (create) new currency units and add them to the total supply
    pub fn mint(&mut self, amount: f64) {
        self.total_supply += amount;
        self.last_issuance = Utc::now();
    }

    // Burn (destroy) currency units, reducing the total supply
    pub fn burn(&mut self, amount: f64) -> Result<(), String> {
        if amount > self.total_supply {
            return Err("Insufficient supply to burn".to_string());
        }
        self.total_supply -= amount;
        Ok(())
    }
}

// ===============================================
// CurrencySystem Struct
// ===============================================
// This struct manages all currencies in our system, providing methods
// for creating, retrieving, and managing multiple currencies.

pub struct CurrencySystem {
    pub currencies: HashMap<CurrencyType, Currency>, // Stores all currencies in the system
}

// Implementation of methods for the CurrencySystem struct
impl CurrencySystem {
    // Create a new CurrencySystem and initialize it with default currencies
    pub fn new() -> Self {
        let mut system = CurrencySystem {
            currencies: HashMap::new(),
        };
        
        // Initialize default currencies with initial supply and issuance rates
        system.add_currency(CurrencyType::BasicNeeds, 1_000_000.0, 0.01);
        system.add_currency(CurrencyType::Education, 500_000.0, 0.005);
        system.add_currency(CurrencyType::Environmental, 750_000.0, 0.008);
        system.add_currency(CurrencyType::Community, 250_000.0, 0.003);
        system.add_currency(CurrencyType::Volunteer, 100_000.0, 0.002);
        system.add_currency(CurrencyType::Storage, 1_000_000.0, 0.01);
        system.add_currency(CurrencyType::Processing, 500_000.0, 0.005);
        system.add_currency(CurrencyType::Energy, 750_000.0, 0.008);
        system.add_currency(CurrencyType::Luxury, 100_000.0, 0.001);
        system.add_currency(CurrencyType::Service, 200_000.0, 0.004);

        system
    }

    // Add a new currency to the system
    pub fn add_currency(&mut self, currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) {
        let currency = Currency::new(currency_type.clone(), initial_supply, issuance_rate);
        self.currencies.insert(currency_type, currency);
    }

    // Get a reference to a currency in the system
    pub fn get_currency(&self, currency_type: &CurrencyType) -> Option<&Currency> {
        self.currencies.get(currency_type)
    }

    // Get a mutable reference to a currency in the system
    pub fn get_currency_mut(&mut self, currency_type: &CurrencyType) -> Option<&mut Currency> {
        self.currencies.get_mut(currency_type)
    }

    // Create a custom currency and add it to the system
    pub fn create_custom_currency(&mut self, name: String, initial_supply: f64, issuance_rate: f64) -> Result<(), String> {
        let currency_type = CurrencyType::Custom(name.clone());
        if self.currencies.contains_key(&currency_type) {
            return Err(format!("Currency '{}' already exists", name));
        }
        self.add_currency(currency_type, initial_supply, issuance_rate);
        Ok(())
    }

    // Perform adaptive issuance for all currencies in the system
    pub fn adaptive_issuance(&mut self) {
        let now = Utc::now();
        for currency in self.currencies.values_mut() {
            let time_since_last_issuance = now.signed_duration_since(currency.last_issuance);
            let issuance_amount = currency.total_supply * currency.issuance_rate * time_since_last_issuance.num_milliseconds() as f64 / 86_400_000.0; // Daily rate
            currency.mint(issuance_amount);
            currency.last_issuance = now;
        }
    }

    // Print the total supply of each currency in the system
    pub fn print_currency_supplies(&self) {
        println!("Currency Supplies:");
        for (currency_type, currency) in &self.currencies {
            println!("{:?}: {}", currency_type, currency.total_supply);
        }
    }
}

// ===============================================
// Wallet Struct
// ===============================================
// This struct represents a user's wallet, capable of holding multiple currencies

pub struct Wallet {
    balances: HashMap<CurrencyType, f64>, // Stores the balance for each currency type
}

// Implementation of methods for the Wallet struct
impl Wallet {
    // Create a new, empty wallet
    pub fn new() -> Self {
        Wallet {
            balances: HashMap::new(),
        }
    }

    // Deposit a specific amount of a currency into the wallet
    pub fn deposit(&mut self, currency_type: CurrencyType, amount: f64) {
        *self.balances.entry(currency_type).or_insert(0.0) += amount;
    }

    // Withdraw a specific amount of a currency from the wallet
    pub fn withdraw(&mut self, currency_type: CurrencyType, amount: f64) -> Result<(), String> {
        let balance = self.balances.entry(currency_type.clone()).or_insert(0.0);
        if *balance < amount {
            return Err(format!("Insufficient balance for {:?}", currency_type));
        }
        *balance -= amount;
        Ok(())
    }

    // Get the balance of a specific currency in the wallet
    pub fn get_balance(&self, currency_type: &CurrencyType) -> f64 {
        *self.balances.get(currency_type).unwrap_or(&0.0)
    }

    // Print the balances of all currencies in the wallet
    pub fn print_balances(&self) {
        println!("Wallet Balances:");
        for (currency_type, balance) in &self.balances {
            println!("{:?}: {}", currency_type, balance);
        }
    }
}

// ===============================================
// Unit Tests
// ===============================================
// These tests verify the correct functionality of our currency system

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_currency_system() {
        let mut system = CurrencySystem::new();
        assert_eq!(system.currencies.len(), 10); // 10 default currencies

        system.create_custom_currency("TestCoin".to_string(), 1000.0, 0.01).unwrap();
        assert_eq!(system.currencies.len(), 11);

        let test_coin = system.get_currency(&CurrencyType::Custom("TestCoin".to_string())).unwrap();
        assert_eq!(test_coin.total_supply, 1000.0);

        // Sleep for a short duration to allow for issuance
        sleep(Duration::from_millis(10));

        system.adaptive_issuance();
        
        // Check if the supply has increased, even if by a small amount
        let basic_needs_supply = system.get_currency(&CurrencyType::BasicNeeds).unwrap().total_supply;
        assert!(basic_needs_supply > 1_000_000.0);

        // Print currency supplies
        system.print_currency_supplies();
    }

    #[test]
    fn test_wallet() {
        let mut wallet = Wallet::new();

        wallet.deposit(CurrencyType::BasicNeeds, 500.0);
        assert_eq!(wallet.get_balance(&CurrencyType::BasicNeeds), 500.0);

        wallet.withdraw(CurrencyType::BasicNeeds, 200.0).unwrap();
        assert_eq!(wallet.get_balance(&CurrencyType::BasicNeeds), 300.0);

        assert!(wallet.withdraw(CurrencyType::BasicNeeds, 400.0).is_err());

        // Print wallet balances
        wallet.print_balances();
    }
}

// ===============================================
// End of File
// ===============================================
// This concludes the implementation of our currency system. It provides
// a flexible and extensible framework for managing multiple currencies
// within our blockchain ecosystem.