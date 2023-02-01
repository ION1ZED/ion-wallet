use std::fs::File;
use std::io::prelude::*;
use crate::wallet_info::*;
use crate::traits::*;

extern crate ring;
use ring::aead::*;

pub fn write_file(filename: &str, data: String){
    let mut file = File::create(filename).expect("cannot create file");
    file.write_all(data.as_bytes()).expect("cannot write to file");
}

pub fn read_file(filename: &str) -> String{
    let mut file = File::open(filename).expect("cannot open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("cannot read file");
    contents
}

pub fn write_wallet(wallet: WalletInfo, password: &str){
    let json = serde_json::to_string(&wallet).unwrap();
    let contents = encrypt(json.as_bytes(), password).unwrap();
    let content_string = contents.to_string();
    write_file("wallet_info.json", content_string);
}

pub fn read_wallet(password: &str) -> Result<WalletInfo, String>{
    let mut file = match File::open("wallet_info.json"){
        Ok(x) => x,
        Err(e) => return Err(format!("cannot open file [Error: {}]", e))
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents){
        Ok(_) => (),
        Err(e) => return Err(format!("cannot read file [Error: {}]", e))
    };
    let decrypted_contents = String::from_utf8(
        decrypt(
            &contents.to_bytes().map_err(|e| format!("corrupt wallet file [Error: {}]", e))?,
            password
        )
        .map_err(|_| format!("invalid password"))?
    )
        .unwrap();
    Ok(serde_json::from_str(&decrypted_contents).map_err(|e| format!("cannot parse JSON from file [Error: {}", e))?)
}

pub fn write_transaction_history(wallet: Vec<TransactionHistory>, password: &str){
    let json = serde_json::to_string(&wallet).unwrap();
    let contents = encrypt(json.as_bytes(), password).unwrap();
    let content_string = contents.to_string();
    write_file("transaction_history.json", content_string);
}

pub fn read_transaction_history(password: &str) -> Result<Vec<TransactionHistory>, String>{
    let mut file = match File::open("transaction_history.json"){
        Ok(x) => x,
        Err(e) => return Err(format!("cannot open file [Error: {}]", e))
    };
    let mut contents = String::new();
    match file.read_to_string(&mut contents){
        Ok(_) => (),
        Err(e) => return Err(format!("cannot read file [Error: {}]", e))
    };
    let decrypted_contents = String::from_utf8(
        decrypt(
            &contents.to_bytes().map_err(|e| format!("corrupt transaction history file [Error: {}]", e))?,
            password
        )
        .map_err(|_| format!("invalid password for transaction history file"))?
    )
        .unwrap();
    Ok(serde_json::from_str(&decrypted_contents).map_err(|e| format!("cannot parse JSON from file [Error: {}", e))?)
}


pub fn write_will_parts(will_parts: (String, String)){
    let (child_will_parts, guardian_will_parts) = will_parts;
    write_will_child(child_will_parts);
    write_will_guardian(guardian_will_parts);
}

pub fn write_will_child(child_will_parts: String){
    write_file("will_parts(Child).json", child_will_parts);
}

pub fn write_will_guardian(guardian_will_parts: String){
    write_file("will_parts(Guardian).json", guardian_will_parts);
}


pub fn write_keys(key: &str, password: &str){
    let key_bytes = key.to_bytes().unwrap();
    let contents = encrypt(&key_bytes, password).unwrap().to_string();
    write_file("wallet_keys.json", contents)
}

pub fn read_keys(password: &str) -> String{
    let mut file = File::open("wallet_keys.json").expect("cannot open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("cannot read file");
    let key_bytes = contents.to_bytes().expect("corrupted key file: key is not a valid hexadecimal string");
    decrypt(&key_bytes, password).expect("invalid password").to_string()
}

fn encrypt(plaintext: &[u8], key: &str) -> Result<Vec<u8>, String> {
    let nonce_data = [124; 12]; // Just an example
    let mut data = plaintext.to_owned();
    
    let key_bytes = key.as_bytes().into_iter().cycle();
    let mut key = [0u8;32];
    key.iter_mut().zip(key_bytes).for_each(|(a,b)| *a = *b);
    
    let key = match UnboundKey::new(&CHACHA20_POLY1305, &key){
        Ok(val) => val,
        Err(err) => return Err(format!("{}",err))
    };
    let key = LessSafeKey::new(key);
    
    // encoding
    let nonce = Nonce::assume_unique_for_key(nonce_data);
    match key.seal_in_place_append_tag(nonce, Aad::empty(), &mut data){
        Ok(_) => (),
        Err(err) => return Err(format!("{}",err))
    };
    Ok(data)
}

fn decrypt(ciphertext: &[u8], key: &str) -> Result<Vec<u8>, String> {
    // Create a new P-256 decryptor
    let nonce_data = [124; 12]; // Just an example
    let mut data = ciphertext.to_owned();

    let key_bytes = key.as_bytes().into_iter().cycle();
    let mut key = [0u8;32];
    key.iter_mut().zip(key_bytes).for_each(|(a,b)| *a = *b);

    let key = match UnboundKey::new(&CHACHA20_POLY1305, &key){
        Ok(val) => val,
        Err(err) => return Err(format!("{}",err))
    };
    let key = LessSafeKey::new(key);
    
    // decoding
    let nonce = Nonce::assume_unique_for_key(nonce_data);
    let data = match key.open_in_place(nonce, Aad::empty(), &mut data){
        Ok(val) => val,
        Err(err) => return Err(format!("{}",err))
    };
    Ok(data.to_owned())
}


