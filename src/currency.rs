use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CurrencyType {
    BasicNeeds,
    Education,
    Environmental,
    Community,
    Volunteer,
}

pub struct Currency {
    pub currency_type: CurrencyType,
    pub total_supply: f64,
}

impl Currency {
    pub fn new(currency_type: CurrencyType, initial_supply: f64) -> Self {
        Currency {
            currency_type,
            total_supply: initial_supply,
        }
    }

    pub fn mint(&mut self, amount: f64) {
        self.total_supply += amount;
    }

    pub fn burn(&mut self, amount: f64) -> Result<(), String> {
        if amount > self.total_supply {
            return Err("Insufficient supply to burn".to_string());
        }
        self.total_supply -= amount;
        Ok(())
    }
}

pub struct CurrencySystem {
    pub currencies: Vec<Currency>,
}

impl CurrencySystem {
    pub fn new() -> Self {
        CurrencySystem {
            currencies: vec![
                Currency::new(CurrencyType::BasicNeeds, 1_000_000.0),
                Currency::new(CurrencyType::Education, 500_000.0),
                Currency::new(CurrencyType::Environmental, 750_000.0),
                Currency::new(CurrencyType::Community, 250_000.0),
                Currency::new(CurrencyType::Volunteer, 100_000.0),
            ],
        }
    }

    pub fn get_currency(&self, currency_type: &CurrencyType) -> Option<&Currency> {
        self.currencies.iter().find(|c| c.currency_type == *currency_type)
    }

    pub fn get_currency_mut(&mut self, currency_type: &CurrencyType) -> Option<&mut Currency> {
        self.currencies.iter_mut().find(|c| c.currency_type == *currency_type)
    }

    pub fn print_currency_supplies(&self) {
        println!("Currency Supplies:");
        for currency in &self.currencies {
            println!("{:?}: {}", currency.currency_type, currency.total_supply);
        }
    }
}