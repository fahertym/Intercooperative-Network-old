use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
// use crate::currency::CurrencyType;
use erased_serde::serialize_trait_object;
use log::{debug, info};

pub trait SmartContract: erased_serde::Serialize {
    fn execute(&self, env: &mut ExecutionEnvironment) -> Result<String, String>;
    fn id(&self) -> String;
}

serialize_trait_object!(SmartContract);

#[derive(Default)]
pub struct ExecutionEnvironment {
    pub state: String,
}

impl ExecutionEnvironment {
    pub fn new() -> Self {
        debug!("Creating new ExecutionEnvironment");
        ExecutionEnvironment {
            state: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssetTokenContract {
    pub asset_id: String,
    pub name: String,
    pub description: String,
    pub owner: String,
    pub value: f64,
}

impl SmartContract for AssetTokenContract {
    fn execute(&self, _env: &mut ExecutionEnvironment) -> Result<String, String> {
        debug!("Executing AssetTokenContract: {}", self.asset_id);
        // Implementation would go here
        info!("AssetTokenContract executed successfully: {}", self.asset_id);
        Ok("Asset token created".to_string())
    }

    fn id(&self) -> String {
        self.asset_id.clone()
    }
}

#[derive(Serialize, Deserialize)]
pub struct BondContract {
    pub bond_id: String,
    pub name: String,
    pub description: String,
    pub issuer: String,
    pub face_value: f64,
    pub maturity_date: DateTime<Utc>,
    pub interest_rate: f64,
    pub owner: String,
}

impl SmartContract for BondContract {
    fn execute(&self, _env: &mut ExecutionEnvironment) -> Result<String, String> {
        debug!("Executing BondContract: {}", self.bond_id);
        // Implementation would go here
        info!("BondContract executed successfully: {}", self.bond_id);
        Ok("Bond created".to_string())
    }

    fn id(&self) -> String {
        self.bond_id.clone()
    }
}

impl AssetTokenContract {
    pub fn new(asset_id: String, name: String, description: String, owner: String, value: f64) -> Self {
        debug!("Creating new AssetTokenContract: {}", asset_id);
        Self {
            asset_id,
            name,
            description,
            owner,
            value,
        }
    }
}

impl BondContract {
    pub fn new(bond_id: String, name: String, description: String, issuer: String, face_value: f64, maturity_date: DateTime<Utc>, interest_rate: f64, owner: String) -> Self {
        debug!("Creating new BondContract: {}", bond_id);
        Self {
            bond_id,
            name,
            description,
            issuer,
            face_value,
            maturity_date,
            interest_rate,
            owner,
        }
    }
}