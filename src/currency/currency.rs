// Filename: src/currency/currency.rs

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
// Importing necessary libraries and modules:
// - chrono: For handling dates and times.
// - std::collections::HashMap: For efficient key-value storage.
// - serde::{Serialize, Deserialize}: For enabling data structure serialization.
// - std::fmt: For formatting traits.
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fmt;

// ===============================================
// CurrencyType Enum
// ===============================================
// An enumeration defining different types of currencies used in our system,
// each representing a specific economic sector or utility.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum CurrencyType {
    BasicNeeds,    // For essential goods and services like food, water.
    Education,     // For educational services and resources.
    Environmental, // For supporting environmental projects and sustainability.
    Community,     // For community enhancement projects.
    Volunteer,     // For recognizing volunteer work.
    Storage,       // For data storage services.
    Processing,    // For computational services.
    Energy,        // For energy trading and services.
    Luxury,        // For luxury goods and services.
    Service,       // For general services not covered by other categories.
    Custom(String), // For user-defined currencies, allowing for extensibility.
}

// Implementing Display trait for CurrencyType to enable easy printing and formatting.
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
// Defines a structure for a single currency, including its type, total supply,
// creation date, last issuance, and issuance rate.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Currency {
    pub currency_type: CurrencyType, // Type of the currency.
    pub total_supply: f64,           // Total amount of currency in circulation.
    pub creation_date: DateTime<Utc>,// Date when the currency was created.
    pub last_issuance: DateTime<Utc>,// Last date when new units were issued.
    pub issuance_rate: f64,          // Rate at which new units are issued (percentage per annum).
}

// Methods associated with the Currency struct.
impl Currency {
    // Constructs a new Currency with specified initial values.
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

    // Minting function to increase the supply of currency.
    pub fn mint(&mut self, amount: f64) {
        self.total_supply += amount;
        self.last_issuance = Utc::now();
    }

    // Burning function to decrease the supply of currency.
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
// Manages all currencies in the system, providing methods for creating,
// retrieving, and managing currencies.
#[derive(Debug, Serialize, Deserialize)]
pub struct CurrencySystem {
    pub currencies: HashMap<CurrencyType, Currency>, // Stores all currencies in the system.
}

// Methods associated with the CurrencySystem struct.
impl CurrencySystem {
    // Initializes a new CurrencySystem with default set of currencies.
    pub fn new() -> Self {
        let mut system = CurrencySystem {
            currencies: HashMap::new(),
        };
        
        // Adding default currencies with initial supplies and issuance rates.
        system.add_currency(CurrencyType::BasicNeeds, 1_000_000.0, 0.01);
        system.add_currency(CurrencyType::Education, 500_000.0, 0.005);
        system.add_currency(CurrencyType::Environmental, 750_000.0, 0.008);
        system.add_currency(CurrencyType::Community, 250_000.0, 0.003);
        system.add_currency(CurrencyType::Volunteer, 100_000.0, 0.002);
        system.add_currency(CurrencyType::Storage, 1_000_000.0, 0.01);
        system.add_currency(CurrencyType::Processing, 500_000.0, 0.005);
        system.add_currency(CurrencyType::Energy, 750_000.0, 0.008);
        system.add_currency(CurrencyType::Luxury, 100_000.0, 0.001);
        system.add_currency(CurrencyType::Service, 200,000.0, 0.004);

        system
    }

    // Adds a new currency to the system.
    pub fn add_currency(&mut self, currency_type: CurrencyType, initial_supply: f64, issuance_rate: f64) {
        let currency = Currency::new(currency_type.clone(), initial_supply, issuance_rate);
        self.currencies.insert(currency_type, currency);
    }

    // Retrieves a reference to a currency by type.
    pub fn get_currency(&self, currency_type: &CurrencyType) -> Option<&Currency> {
        self.currencies.get(currency_type)
    }

    // Retrieves a mutable reference to a currency for modification.
    pub fn get_currency_mut(&mut self, currency_type: &CurrencyType) -> Option<&mut Currency> {
        self.currencies.get_mut(currency_type)
    }

    // Adds a new custom currency to the system.
    pub fn create_custom_currency(&mut self, name: String, initial_supply: f64, issuance_rate: f64) -> Result<(), String> {
        let currency_type = CurrencyType::Custom(name.clone());
        if self.currencies.contains_key(&currency_type) {
            return Err(format!("Currency '{}' already exists", name));
        }
        self.add_currency(currency_type, initial_supply, issuance_rate);
        Ok(())
    }

    // Method to adjust issuance for all currencies based on defined rates.
    pub fn adaptive_issuance(&mut self) {
        let now = Utc::now();
        for currency in self.currencies.values_mut() {
            let time_since_last_issuance = now.signed_duration_since(currency.last_issuance);
            let issuance_amount = currency.total_supply * currency.issuance_rate * time_since_last_issuance.num_milliseconds() as f64 / 86_400_000.0; // Daily rate
            currency.mint(issuance_amount);
            currency.last_issuance = now;
        }
    }

    // Prints the total supply of each currency.
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
// Represents a wallet for a user, storing balances for each currency type.
#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    balances: HashMap<CurrencyType, f64>, // Map of currency types to their balances.
}

// Methods associated with the Wallet struct.
impl Wallet {
    // Creates a new empty wallet.
    pub fn new() -> Self {
        Wallet {
            balances: HashMap::new(),
        }
    }

    // Deposits a specific amount of a currency into the wallet.
    pub fn deposit(&mut self, currency_type: CurrencyType, amount: f64) {
        *self.balances.entry(currency_type).or_insert(0.0) += amount;
    }

    // Withdraws a specific amount of a currency from the wallet, if sufficient balance exists.
    pub fn withdraw(&mut self, currency_type: CurrencyType, amount: f64) -> Result<(), String> {
        let balance = self.balances.entry(currency_type.clone()).or_insert(0.0);
        if *balance < amount {
            return Err(format!("Insufficient balance for {:?}", currency_type));
        }
        *balance -= amount;
        Ok(())
    }

    // Retrieves the balance for a specific currency.
    pub fn get_balance(&self, currency_type: &CurrencyType) -> f64 {
        *self.balances.get(currency_type).unwrap_or(&0.0)
    }

    // Prints balances of all currencies in the wallet.
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
// Provides tests to ensure functionality of our currency system.
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
