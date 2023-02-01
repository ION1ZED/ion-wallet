use std::u64;

use dotenv;
use reqwest;
use tokio;
use serde_json::Result;

use crate::blockchain_status::BlockchainStatus;
use crate::blockchain_address::BlockchainAddress;
use crate::blockchain_transaction::BlockchainTransaction;
use crate::blockchain_utxo::UTXO;
use crate::wallet_info::*;


const HOST_ROOT: &str = "https://btcbook.nownodes.io/api/";
const HOST_ROOT_TESTNET: &str = "https://btcbook-testnet.nownodes.io/api/";

#[tokio::main]
pub async fn send_request(url: &str) -> String{
    
    let client = reqwest::Client::new();

    client
        .get(url)
        .header("api-key", dotenv::var("API_KEY").expect("No API_KEY found"))
        .send()
        .await
        .expect("Failed to get response...")
        .text()
        .await
        .expect("Failed to convert payload...")
}

pub fn blockchain_status_request() -> BlockchainStatus{
    let response = send_request(HOST_ROOT);
    serde_json::from_str(&response).expect("cannot parse Blockchain Status JSON")
}

pub fn blockchain_address_request(address: &str) -> BlockchainAddress{
    let url: String = String::new() + HOST_ROOT + "v2/address/" + address;
    let response = send_request(&url);
    serde_json::from_str(&response).expect("cannot parse Blockchain Address JSON")
}

pub fn blockchain_transaction_request(txid: &str) -> BlockchainTransaction{
    let url: String = String::new() + HOST_ROOT + "v2/tx/" + txid;
    let response = send_request(&url);
    serde_json::from_str(&response).expect("cannot parse Blockchain Transaction JSON")
}

pub fn blockchain_utxo_request(address: &str) -> Vec<UTXO>{
    let url: String = String::new() + HOST_ROOT + "v2/utxo/" + address;
    let response = send_request(&url);
    serde_json::from_str(&response).expect("cannot parse Blockchain UTXO JSON")
}

pub fn get_address_balance(address: &str) -> u64{
    let mut address_balance: i64 = 0;
    let blockchain_address = blockchain_address_request(address);
    // let mut i = 0;
    for txid in blockchain_address.txids{
        let transaction = blockchain_transaction_request(&txid);
        address_balance += get_transaction_value(transaction, address);
    }
    address_balance as u64
}

pub fn get_transaction_value(transaction: BlockchainTransaction, address: &str) -> i64{
    let address = address.to_string();
    let mut amount_transacted: i64 = 0;
    for vin in transaction.vin{
        if vin.addresses[0] == address {
            amount_transacted -= vin.value.parse::<i64>().unwrap();
        }
    }
    for vout in transaction.vout{
        if vout.addresses[0] == address {
            amount_transacted += vout.value.parse::<i64>().unwrap();
        }
    }
    amount_transacted
}






pub fn testnet_status_request() -> BlockchainStatus{
    let response = send_request(HOST_ROOT_TESTNET);
    serde_json::from_str(&response).expect("cannot parse Blockchain Status JSON")
}

pub fn testnet_address_request(address: &str) -> BlockchainAddress{
    let url: String = String::new() + HOST_ROOT_TESTNET + "v2/address/" + address;
    let response = send_request(&url);
    serde_json::from_str(&response).expect("cannot parse Blockchain Address JSON")
}

pub fn testnet_transaction_request(txid: &str) -> BlockchainTransaction{
    let url: String = String::new() + HOST_ROOT_TESTNET + "v2/tx/" + txid;
    let response = send_request(&url);
    serde_json::from_str(&response).expect("cannot parse Blockchain Transaction JSON")
}

pub fn testnet_utxo_request(address: &str) -> Vec<UTXO>{
    let url: String = String::new() + HOST_ROOT_TESTNET + "v2/utxo/" + address;
    let response = send_request(&url);
    serde_json::from_str(&response).expect("cannot parse Blockchain UTXO JSON")
}

pub fn testnet_broadcast_transaction(transaction_raw_hex: &str) -> String{
    let url: String = String::new() + HOST_ROOT_TESTNET + "v2/sendtx/" + transaction_raw_hex;
    let response = send_request(&url);
    response
}




pub fn testnet_address_history(address: &str, n: usize) -> Vec<TransactionHistory>{
    let mut result: Vec<TransactionHistory> = Vec::new();
    let blockchain_address = testnet_address_request(address);
    for i in 0..n{
        let transaction = testnet_transaction_request(&blockchain_address.txids[i]);
        let confs = transaction.confirmations;
        let mut current = get_transaction_history(transaction, address);
        current.confirmations = confs;
        result.push(current)
    }
    result
}

pub fn get_transaction_history(transaction: BlockchainTransaction, address: &str) -> TransactionHistory{
    let address = address.to_string();
    let mut to_address: String = String::new();
    let mut to: bool = true;
    let mut value: u64 = 0;
    for vin in transaction.vin.clone(){
        if vin.addresses[0] == address {
            to = true;
            if transaction.vout.len() < 3{
                for vout in transaction.vout.clone(){
                    to_address = vout.addresses[0].clone();
                }
            }else{
                to_address = format!("{} outputs...", transaction.vout.len());
            }
            value = vin.value.parse::<u64>().unwrap();
            return TransactionHistory::new(to_address, to, value, 0)
        }
    }
    for vout in transaction.vout{
        if vout.addresses[0] == address {
            to = false;
            if transaction.vin.len() < 3{
                for vin in transaction.vin.clone(){
                    to_address = vin.addresses[0].clone();
                }
            }else{
                to_address = format!("{} inputs...", transaction.vin.len());
            }
            value = vout.value.parse::<u64>().unwrap();
        }
    }
    TransactionHistory::new(to_address, to, value, 0)
}