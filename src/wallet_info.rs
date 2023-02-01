use serde::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalletInfo{
    pub pubkey: String,
    pub address: String,
    pub value: u64,
    pub inheritors: Vec<Inheritor>,
    pub guardians: Vec<Guardian>,
    pub locktime: u32,
}

impl WalletInfo{
    pub fn new(pubkey: String, address: String, value: u64, inheritors: Vec<Inheritor>, guardians: Vec<Guardian>, locktime: u32) -> Self{
        WalletInfo {
            pubkey,
            address,
            value,
            inheritors,
            guardians,
            locktime,
        }    
    }

    pub fn new_empty() -> Self{
        WalletInfo {
            pubkey: String::new(),
            address: String::new(),
            value: 0,
            inheritors: vec![],
            guardians: vec![],
            locktime: 0,
        }    
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Inheritor{
    pub name: String,
    pub address: String,
    pub id: String,
    pub value: u64,
}

impl Inheritor{
    pub fn new(name: String, address: String, id: String, value: u64) -> Self {
        Inheritor{
            name,
            address,
            id,
            value,
        }
    }
}

pub trait Inheritors{
    fn addresses(&self) -> Vec<String>;
    fn amounts(&self) -> Vec<u64>;
}

impl Inheritors for Vec<Inheritor>{
    //returns a vector containing the addresses of each inheritor
    fn addresses(&self) -> Vec<String>{
        let mut result = Vec::new();
        for inheritor in self{
            result.push(inheritor.address.clone());
        }
        result
    }

    //returns a vector containing the amount of satoshis being inherited by each inheritor
    fn amounts(&self) -> Vec<u64>{
        let mut result = Vec::new();
        for inheritor in self{
            result.push(inheritor.value);
        }
        result
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Guardian{
    pub name: String,
    pub id: String,
}

impl Guardian{
    pub fn new(name: String, id: String) -> Self {
        Guardian{
            name,
            id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WalletKeys{
    pub privkey: String,
}

impl WalletKeys{
    pub fn new(privkey: String) -> Self{
        WalletKeys {
            privkey,
        }    
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionHistory{
    pub address: String,
    pub to: bool,
    pub value: u64,
    pub confirmations: u64,
}

impl TransactionHistory{
    pub fn new(address: String, to: bool, value: u64, confirmations: u64) -> Self {
        TransactionHistory{
            address,
            to,
            value,
            confirmations,
        }
    }
}