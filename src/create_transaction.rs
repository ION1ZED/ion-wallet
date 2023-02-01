use std::str;
use std::u8;

use num_bigint::*;
use num_traits::{Zero, One};

use crate::blockchain_info;
use crate::blockchain_info::*;
use crate::blockchain_status::BlockchainStatus;
use crate::blockchain_address::BlockchainAddress;
use crate::blockchain_transaction::BlockchainTransaction;
use crate::blockchain_utxo::UTXO;
use crate::transaction_parts::*;
use crate::will_components::TimelockComponents;
use crate::traits::*;

use bitcoin::util::hash::{Sha256dHash, Hash160};

use secp256k1::rand::rngs::*;
use secp256k1::Secp256k1;
use secp256k1::*;
use sha2::{Sha256, Digest};

use std::str::FromStr;
use std::thread;


const MAX32: u32 = 4294967295;


pub fn create_transaction(to_address: &str, to_value: u64, fee: u64, my_address: &str, my_redeem_script: &str, secret_key: SecretKey) -> Result<SignedTransaction, String>{
    let secp = Secp256k1::new();
    let mut has_segwit = false;
    let (destination_locking_script,output_is_segwit) = decode_address(to_address)?;

    if output_is_segwit{
        has_segwit = true;
    }

    let (my_locking_script,_) = decode_address(my_address)?;

    //create transaction inputs
    let mut input_total: u64 = 0;
    let mut vins: Vec<Vin> = Vec::new();
    
    let utxos: Vec<UTXO> = blockchain_info::testnet_utxo_request(my_address);
    for utxo in utxos{
        let transaction: BlockchainTransaction = blockchain_info::testnet_transaction_request(&utxo.txid);
        let vin = Vin::new(&utxo.txid, utxo.vout, &transaction.vout[utxo.vout as usize].hex, vec![my_redeem_script], MAX32, utxo.value.parse::<u64>().unwrap())?;
        input_total += utxo.value.parse::<u64>().unwrap();
        vins.push(vin);
        if input_total > to_value + fee{
            break;
        }
    }

    //create transaction output to destination
    let mut vouts: Vec<Vout> = Vec::new();
    vouts.push(Vout::new(to_value, &destination_locking_script)?);

    //return change
    if let Some(subtotal) = to_value.checked_add(fee) {
        if subtotal < input_total {
            vouts.push(Vout::new(input_total - subtotal, &my_locking_script)?);
        }else if subtotal > input_total{
            return Err(String::from("Not enough coins"));
        }
    }else{
        return Err(String::from(""));
    }

    //create unsigned transaction
    let mut raw_transaction: RawTransaction = RawTransaction::new(2,vins.clone(),vouts,0);

    //sign transaction
    for i in 0..vins.len(){
        let legacy_unsigned_transaction = raw_transaction.clone().concat_legacy(i, 1).to_string();

        //create legacy signatures
        let message = Message::from_slice(&sha256d(&legacy_unsigned_transaction)).unwrap();
        let signature = secp.sign_ecdsa(&message, &secret_key);

        raw_transaction.vins[i].sign( &format!("{}", signature), 1);
    }
    
    //combine all for final transaction
    let witnesses = vec![];
    let signed = SignedTransaction::new(raw_transaction,witnesses,has_segwit);
    Ok(signed)
}

pub fn predict_will_parts(child_addresses: Vec<String>, child_amounts: Vec<u64>, locktime_blocks: u16, previous_transaction: SignedTransaction, parent_address: &str, parent_pubkey: &str, parent_secretkey: SecretKey) -> (String, String){

    let mut child_will_parts = String::new();
    
    let timelock = generate_timelock_components(parent_pubkey, locktime_blocks);
    let will_initiation = predict_will_initiation(previous_transaction, parent_address, parent_secretkey, &timelock.locking_script.to_string(), parent_pubkey, 500).unwrap();
    child_will_parts.push_str(&format!("Will Initiation: {}\n\n",will_initiation.clone().concat().to_string()));
    let will_redemption = create_will_redemption(will_initiation.clone(), timelock.clone(), child_amounts, child_addresses).unwrap();
    child_will_parts.push_str(&format!("Will Redemption: {}\n\n",will_redemption.clone().concat().to_string()));

    let will_revocation = create_will_revocation(parent_secretkey, will_initiation.clone(), timelock, parent_address, 250).unwrap();
    let guardian_will_parts = format!("Will Revocation: {}",will_revocation.concat().to_string());

    (child_will_parts, guardian_will_parts)
}

pub fn create_will_parts(child_addresses: Vec<String>, child_amounts: Vec<u64>, locktime_blocks: u16, parent_address: &str, parent_pubkey: &str, parent_secretkey: SecretKey) -> (String, String){

    let mut child_will_parts = String::new();
    
    let timelock = generate_timelock_components(parent_pubkey, locktime_blocks);
    let will_initiation = create_will_initiation(parent_address, parent_secretkey, &timelock.locking_script.to_string(), parent_pubkey, 500).unwrap();
    child_will_parts.push_str(&format!("Will Initiation: {}\n\n",will_initiation.clone().concat().to_string()));
    let will_redemption = create_will_redemption(will_initiation.clone(), timelock.clone(), child_amounts, child_addresses).unwrap();
    child_will_parts.push_str(&format!("Will Redemption: {}\n\n",will_redemption.clone().concat().to_string()));

    let will_revocation = create_will_revocation(parent_secretkey, will_initiation.clone(), timelock, parent_address, 250).unwrap();
    let guardian_will_parts = format!("Will Revocation: {}",will_revocation.concat().to_string());

    (child_will_parts, guardian_will_parts)
}

pub fn predict_will_initiation(prev_transaction: SignedTransaction, my_address: &str, secret_key: SecretKey, timelock_locking_script: &str, my_redeem_script: &str, fee: u64) -> Result<SignedTransaction, String>{
    let secp = Secp256k1::new();
    
    //create transaction inputs
    let mut input_satoshis: u64 = 0;
    let mut vins: Vec<Vin> = Vec::new();
    let (my_locking_script, wallet_is_segwit) = decode_address(my_address)?;
    let prev_txid = sha256d(&prev_transaction.clone().concat().to_string()).reverse().to_string();

    let mut consumed_inputs = Vec::new();
    for vin in prev_transaction.clone().vins{
        consumed_inputs.push(vin.txid.reverse().to_string().to_uppercase());
    }

    for (i, vout) in prev_transaction.vouts.iter().enumerate(){
        if vout.locking_script == my_locking_script.to_bytes()?{
            let input_sat = vout.value.to_int_le();
            vins.push(Vin::new(&prev_txid, i as u32, &my_locking_script, vec![my_redeem_script], MAX32, input_sat)?);
            input_satoshis += input_sat;
        }
    }
    let utxos: Vec<UTXO> = blockchain_info::testnet_utxo_request(my_address);
    
    for utxo in utxos.iter().filter(|&x| !consumed_inputs.contains(&x.txid.to_uppercase())){
        let transaction: BlockchainTransaction = blockchain_info::testnet_transaction_request(&utxo.txid);
        let input_sat = utxo.value.parse::<u64>().unwrap();
        let vin = Vin::new(&utxo.txid, utxo.vout, &transaction.vout[utxo.vout as usize].hex, vec![my_redeem_script], MAX32, input_sat)?;
        input_satoshis += input_sat;
        vins.push(vin);
    }

    //create transaction output to timelock vault
    let satoshis = input_satoshis - fee;
    let mut vouts: Vec<Vout> = vec![Vout::new(satoshis, timelock_locking_script)?];

    //create unsigned transaction
    let mut raw_transaction: RawTransaction = RawTransaction::new(2,vins.clone(),vouts,0);

    //sign transaction
    for (i, _) in vins.iter().enumerate(){
        let legacy_unsigned_transaction = raw_transaction.clone().concat_legacy(i, 1).to_string();

        //create legacy signatures
        let message = Message::from_slice(&sha256d(&legacy_unsigned_transaction)).unwrap();
        let signature = secp.sign_ecdsa(&message, &secret_key);
        raw_transaction.vins[i].sign( &format!("{}", signature), 1);
    }
    
    //combine all for final transaction
    let witnesses = vec![];
    let signed = SignedTransaction::new(raw_transaction,witnesses,wallet_is_segwit);
    Ok(signed)
}


pub fn create_will_initiation(my_address: &str, secret_key: SecretKey, timelock_locking_script: &str, my_redeem_script: &str, fee: u64) -> Result<SignedTransaction, String>{
    let secp = Secp256k1::new();
    
    //create transaction inputs
    let mut input_satoshis: u64 = 0;
    let mut vins: Vec<Vin> = Vec::new();
    let (_,wallet_is_segwit) = decode_address(my_address)?;
    
    let utxos: Vec<UTXO> = blockchain_info::testnet_utxo_request(my_address);
    // let utxo = utxos[1].clone();
    for utxo in utxos{
        let transaction: BlockchainTransaction = blockchain_info::testnet_transaction_request(&utxo.txid);
        let vin = Vin::new(&utxo.txid, utxo.vout, &transaction.vout[utxo.vout as usize].hex, vec![my_redeem_script], MAX32, utxo.value.parse::<u64>().unwrap())?;
        input_satoshis += utxo.value.parse::<u64>().unwrap();
        vins.push(vin);
    }

    //create transaction output to timelock vault
    let satoshis = input_satoshis - fee;
    let mut vouts: Vec<Vout> = vec![Vout::new(satoshis, timelock_locking_script)?];

    //create unsigned transaction
    let mut raw_transaction: RawTransaction = RawTransaction::new(2,vins.clone(),vouts,0);

    //sign transaction
    for (i, _) in vins.iter().enumerate(){
        let legacy_unsigned_transaction = raw_transaction.clone().concat_legacy(i, 1).to_string();

        //create legacy signatures
        let message = Message::from_slice(&sha256d(&legacy_unsigned_transaction)).unwrap();
        let signature = secp.sign_ecdsa(&message, &secret_key);
        raw_transaction.vins[i].sign( &format!("{}", signature), 1);
    }
    
    //combine all for final transaction
    let witnesses = vec![];
    let signed = SignedTransaction::new(raw_transaction,witnesses,wallet_is_segwit);
    Ok(signed)
}

pub fn create_will_redemption(will_initiation: SignedTransaction, timelock_vault: TimelockComponents, child_amounts: Vec<u64>, child_addresses: Vec<String>) -> Result<SignedTransaction, String>{
    let secp = Secp256k1::new();

    //create transaction inputs
    let mut vins: Vec<Vin> = Vec::new();
    // Reverse TXID
    vins.push(Vin::new(&sha256d(&will_initiation.clone().concat_legacy().to_string()).reverse().to_string(), 0,
    &timelock_vault.locking_script.to_string(), vec![&timelock_vault.witness_script.to_string()],
    bytes_le_to_int(timelock_vault.sequence()) as u32, bytes_le_to_int(will_initiation.vouts[0].value.to_vec()))?);

    //create transaction outputs
    let mut vouts: Vec<Vout> = Vec::new();
    for (i, val) in child_amounts.iter().enumerate(){
        let (out_script,_) = decode_address(&child_addresses[i])?;
        vouts.push(Vout::new(child_amounts[i], &out_script)?);
    }

    let mut raw_transaction: RawTransaction = RawTransaction::new(2,vins.clone(),vouts,0);

    //create segwit signatures
    let mut witnesses: Vec<Option<Witness>> = Vec::new();
    let mut unsigned: UnsignedSegwitTransaction = UnsignedSegwitTransaction::new(raw_transaction.clone(), 0, 1);
    unsigned.change_vin_p2wsh(vins[0].clone(), 1, 0);
    //TEST

    let message = Message::from_slice(&unsigned.clone().concat().sha256d()).unwrap();
    let signature= secp.sign_ecdsa(&message, &timelock_vault.single_use_private_key);

    //push empty witness for #1 p2pk transaction
    witnesses.push(Some(Witness::new(vec![&format!("{}", signature)],
    vec![StackItem::OP([0u8]), vins[0].redeem_script[0].clone()], 1)?));

    //combine all for final transaction
    let signed = SignedTransaction::new(raw_transaction,witnesses,true);
    Ok(signed)
}

pub fn create_will_revocation(parent_secretkey: SecretKey, will_initiation: SignedTransaction, timelock_vault: TimelockComponents, return_address: &str, fee: u64) -> Result<SignedTransaction, String>{
    let secp = Secp256k1::new();
    let input_satoshis = bytes_le_to_int(will_initiation.vouts[0].value.to_vec());

    //create transaction inputs
    let mut vins: Vec<Vin> = Vec::new();
    // Reverse TXID
    vins.push(Vin::new(&sha256d(&will_initiation.clone().concat_legacy().to_string()).reverse().to_string(), 0,
    &timelock_vault.locking_script.to_string(), vec![&timelock_vault.witness_script.to_string()],
    4294967295, input_satoshis)?);

    //create transaction outputs
    let mut vouts: Vec<Vout> = Vec::new();
    let (refund_script,_) = decode_address(return_address)?;
    vouts.push(Vout::new(input_satoshis - fee, &refund_script)?);

    let mut raw_transaction: RawTransaction = RawTransaction::new(2,vins.clone(),vouts,0);

    //create segwit signatures
    let mut witnesses: Vec<Option<Witness>> = Vec::new();
    let mut unsigned: UnsignedSegwitTransaction = UnsignedSegwitTransaction::new(raw_transaction.clone(), 0, 1);
    unsigned.change_vin_p2wsh(vins[0].clone(), 1, 0);

    let message = Message::from_slice(&unsigned.clone().concat().sha256d()).unwrap();
    let signature= secp.sign_ecdsa(&message, &parent_secretkey);

    //push empty witness for #1 p2pk transaction
    witnesses.push(Some(Witness::new(vec![&format!("{}", signature)],
    vec![StackItem::OP([1u8]), vins[0].redeem_script[0].clone()], 1)?)); // OP_True = 0x51 = 81

    //combine all for final transaction
    let signed = SignedTransaction::new(raw_transaction,witnesses,true);
    Ok(signed)
}

//generates the single-use public and private keys, as well as bitcoin address of timelock vault
pub fn generate_timelock_components (parent_pubkey: &str, locktime_blocks: u16) -> TimelockComponents{
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    let locktime = if locktime_blocks >= 1 && locktime_blocks <= 16{
        format!("{:X}", (80 + locktime_blocks))
    }else{
        int_to_bytes_le(locktime_blocks).to_string().varint()
    };
    let witness_script = script(["OP_IF", &parent_pubkey.varint(),  "OP_checksig", "OP_ELSE", &locktime, "OP_CHECKSEQUENCEVERIFY", "OP_DROP", &public_key.to_string().varint(), "OP_checksig", "OP_ENDIF"].to_vec());
    let locking_script = wrap_p2wsh(witness_script.to_bytes().unwrap());
    
    let components = TimelockComponents::new(secret_key, public_key, locktime_blocks, witness_script.to_bytes().unwrap(), locking_script).unwrap();
    components
}

pub fn generate_new_wallet (){
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    println!("New Secrey Key: {:?} \n\nNew Public Key: {:?} \n\n", SecretKey::display_secret(&secret_key), public_key.to_string());
    println!("Address: {}", wrap_p2pkh_testnet(&public_key.to_string()));
}

pub fn wrap_p2wsh(redeem_script: Vec<u8>) -> Vec<u8>{
    let mut result = vec![0u8];
    let mut script_hash = redeem_script.sha256();
    result.append(&mut varint(script_hash.len()));
    result.append(&mut script_hash);
    result
}

pub fn wrap_p2pkh(pubkey: &str) -> String{
    let mut first = vec![0u8];
    first.append(&mut hash160(pubkey));
    first.append(&mut sha256d(&first.to_string())[0..4].to_vec());
    hex_to_base58(&first.to_string())
}

pub fn wrap_p2pkh_testnet(pubkey: &str) -> String{
    let mut first = vec![111u8];
    first.append(&mut hash160(pubkey));
    first.append(&mut sha256d(&first.to_string())[0..4].to_vec());
    hex_to_base58(&first.to_string())
}

pub fn decode_address(address: &str) -> Result<(String, bool), String>{
    address_to_lockingscript(&base58_to_hex(address)?)
}

//takes in address, and returns locking script to use in transaction output, and a bool representing whether the transaction is segwit or not
pub fn address_to_lockingscript(address: &str) -> Result<(String, bool), String>{
    let (first_byte,address) = address.split_at(2);
    
    match first_byte{
        "00" => return Ok((script(vec!["OP_dup", "OP_hash160", &remove_checksum("00", address)?.varint(), "OP_equalverify", "OP_checksig"]),false)), //p2pkh
        "6F" => return Ok((script(vec!["OP_dup", "OP_hash160", &remove_checksum("6F", address)?.varint(), "OP_equalverify", "OP_checksig"]),false)), //p2pkh testnet
        "02" => return Ok(((String::from("02") + address).varint() + &script(vec!["OP_checksig"]),false)), //compressed pubkey, positive y
        "03" => return Ok(((String::from("03") + address).varint() + &script(vec!["OP_checksig"]),false)), //compressed pubkey, negative y
        "04" => return Ok(((String::from("04") + address).varint() + &script(vec!["OP_checksig"]),false)), //uncompressed pubkey
        "05" => return Ok((script(["OP_hash160", &remove_checksum("05", address)?.varint(), "OP_equal"].to_vec()),false)), //p2sh
        _ => return Err(format!("Parse Address Error"))
    }
}

pub fn remove_checksum(prefix: &str, address: &str) -> Result<String, String>{
    let (address,checksum) = address.split_at(40);
    if sha256d(&(prefix.to_string() + address))[0..4].to_string() == checksum.to_string(){
        return Ok(address.to_string());
    }else{
        return Err(format!("Address Checksum Failed. Please check that the input address is correct"));
    }
}

pub fn script (script: Vec<&str>) -> String{
    let result: Vec<&str> = script.iter().map(|&word| {
        match word.to_ascii_uppercase().as_ref() {
            "OP_FALSE" => "00",
            "OP_0" => "00",
            "OP_PUSHDATA1" => "4c",
            "OP_PUSHDATA2" => "4d",
            "OP_PUSHDATA4" => "4e",
            "OP_1NEGATE" => "4f",
            "OP_TRUE" => "51",
            "OP_1" => "51",
            "OP_2" => "52",
            "OP_3" => "53",
            "OP_4" => "54",
            "OP_5" => "55",
            "OP_6" => "56",
            "OP_7" => "57",
            "OP_8" => "58",
            "OP_9" => "59",
            "OP_10" => "5a",
            "OP_11" => "5b",
            "OP_12" => "5c",
            "OP_13" => "5d",
            "OP_14" => "5e",
            "OP_15" => "5f",
            "OP_16" => "60",
            "OP_NOP" => "61",
            "OP_IF" => "63",
            "OP_NOTIF" => "64",
            "OP_ELSE" => "67",
            "OP_ENDIF" => "68",
            "OP_VERIFY" => "69",
            "OP_RETURN" => "6a",
            "OP_TOALTSTACK" => "6b",
            "OP_FROMALTSTACK" => "6c",
            "OP_IFDUP" => "73",
            "OP_DEPTH" => "74",
            "OP_DROP" => "75",
            "OP_DUP" => "76",
            "OP_NIP" => "77",
            "OP_OVER" => "78",
            "OP_PICK" => "79",
            "OP_ROLL" => "7a",
            "OP_ROT" => "7b",
            "OP_SWAP" => "7c",
            "OP_TUCK" => "7d",
            "OP_2DROP" => "6d",
            "OP_2DUP" => "6e",
            "OP_3DUP" => "6f",
            "OP_2OVER" => "70",
            "OP_2ROT" => "71",
            "OP_2SWAP" => "72",
            "OP_SIZE" => "82",
            "OP_EQUAL" => "87",
            "OP_EQUALVERIFY" => "88",
            "OP_1ADD" => "8b",
            "OP_1SUB" => "8c",
            "OP_NEGATE" => "8f",
            "OP_ABS" => "90",
            "OP_NOT" => "91",
            "OP_0NOTEQUAL" => "92",
            "OP_ADD" => "93",
            "OP_SUB" => "94",
            "OP_BOOLAND" => "9a",
            "OP_BOOLOR" => "9b",
            "OP_NUMEQUAL" => "9c",
            "OP_NUMEQUALVERIFY" => "9d",
            "OP_NUMNOTEQUAL" => "9e",
            "OP_LESSTHAN" => "9f",
            "OP_GREATERTHAN" => "a0",
            "OP_LESSTHANOREQUAL" => "a1",
            "OP_GREATERTHANOREQUAL" => "a2",
            "OP_MIN" => "a3",
            "OP_MAX" => "a4",
            "OP_WITHIN" => "a5",
            "OP_RIPEMD160" => "a6",
            "OP_SHA1" => "a7",
            "OP_SHA256" => "a8",
            "OP_HASH160" => "a9",
            "OP_HASH256" => "aa",
            "OP_CODESEPARATOR" => "ab",
            "OP_CHECKSIG" => "ac",
            "OP_CHECKSIGVERIFY" => "ad",
            "OP_CHECKMULTISIG" => "ae",
            "OP_CHECKMULTISIGVERIFY" => "af",
            "OP_CHECKLOCKTIMEVERIFY" => "b1",
            "OP_CHECKSEQUENCEVERIFY" => "b2",
            _ => word,
        }
    }).collect();
    result.concat()
}

pub fn hex_to_base58(hex: &str) -> String{
    let mut answer: String = String::new();
    let mut j: usize = 0;
    for (i,char) in hex.chars().enumerate(){
        if char != '0'{
            j = i/2;
            break;
        }
    };
    for _ in 0..j{
        answer.push('1');
    }
    let answer = answer + &int_to_base58(hex_to_int(hex).unwrap());
    answer
}

//converts a hex string to int
pub fn hex_to_int (hexinput: &str) -> Result<BigUint, String>{
    let mut result: BigUint = Zero::zero();
    for char in hexinput.chars(){
        result = result*16u8 + char_to_int(char).unwrap();
    }
    Ok(result)
}

//converts a single hex char (0-F) to a number (0-15)
pub fn char_to_int (c: char) -> Result<u8, String>{
    let hex = c.to_lowercase().next().unwrap();
    match hex.to_digit(16){
        Some(num) => Ok(num as u8),
        None => Err(String::from("invalid hexadecimal digit"))
    }
}

//converts an integer to base58 encoded string
pub fn int_to_base58(int: BigUint) -> String {
    const DIGITS: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let radix = int_from_string("58");
    let mut result = String::new();
    let mut num = int;
    while &num > &Zero::zero() {
        let i: usize = (&num % &radix).try_into().unwrap();
        result.push(DIGITS[i] as char);
        num /= &radix;
    }

    // Add leading zeros
    while result.len() < 8 {
        result.push('1');
    }

    result.chars().rev().collect()
}

//creates a big integer from a string
pub fn int_from_string (a: &str) -> BigUint{
    let mut b = a;
    let mut c: BigUint = Zero::zero();
    for char in b.chars(){
        c = c*10u8 + char.to_digit(10).unwrap();
    }
    return c;
}

//decodes base58 into integer
pub fn base58_to_int(base58: &str) -> Result<BigUint, String> {
    const DIGITS: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    
    let mut result: BigUint = Zero::zero();
    let mut power: BigUint = One::one();
    for c in base58.chars().rev() {
        let digit: BigUint = BigUint::from(DIGITS.iter().position(|&d| d as char == c).ok_or(format!("Invalid character '{}' found in base58 string", c))?);
        result += digit * &power;
        power *= 58u8;
    }
    
    Ok(result)
}

//convert int to hex string
pub fn base58_to_hex(base58: &str) -> Result<String, String>{
    let mut answer: String = String::new();
    let mut j: usize = 0;
    for (i,char) in base58.chars().enumerate(){
        if char != '1'{
            j = i*2;
            break;
        }
    };
    for _ in 0..j{
        answer.push('0');
    }
    let answer = answer + &base58_to_int(base58)?.to_bytes_be().to_string();
    Ok(answer)
}

//creates a sha256d (sha256 + sha256) hash of hex string
pub fn sha256d(hex_string: &str) -> Vec<u8>{
    Sha256dHash::data(&Sha256dHash::from_data(&hex_string.to_bytes().unwrap())).to_vec()
}

//creates a sha256 hash of hex string
pub fn sha256(hex_string: &str) -> Vec<u8>{
    let mut hasher = Sha256::default();
    hasher.input(&hex_string.to_bytes().unwrap());
    let output = hasher.result();
    output.to_vec()
}

//creates a hash160 (sha256 + ripemd160) hash of hex string
pub fn hash160(hex_string: &str) -> Vec<u8>{
    Hash160::data(&Hash160::from_data(&hex_string.to_bytes().unwrap())).to_vec()
}

//convert a little endian byte vector to integer
pub fn bytes_le_to_int(bytes: Vec<u8>) -> u64 {
    let mut num: u64 = 0;
    for (i, byte) in bytes.iter().enumerate() {
        num += (*byte as u64) << (8 * i);
    }
    num
}

pub fn int_to_bytes_le(num: u16) -> Vec<u8>{
    let mut num = num;
    let mut result: Vec<u8> = Vec::new();
    let pow = 256;
    while num > 0 {
        result.push((num % pow) as u8);
        num /= 256;
    }
    result
}
