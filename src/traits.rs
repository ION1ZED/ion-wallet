use sha2::{Sha256, Digest};

pub trait HexString {
    fn to_bytes(&self) -> Result<Vec<u8>, String>;
    fn to_padded_bytes(&self, len: usize) -> Result<Vec<u8>, String>;
    fn reverse_hex (&self) -> Result<String, String>;
    fn varint(&self) -> String;
    fn shorten(&self, len: usize) -> String;
}

impl HexString for String{
    //converts a hex string to hex vector
    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        if self.chars().count() % 2 == 1{
            return Err(format!("Hex strings must have an even number of characters. Your hex string has {} characters.", self.chars().count()))
        }
        let mut vec = Vec::new();
        for i in 0..self.len()/2 {
            let byte = u8::from_str_radix(&self[i*2..i*2+2], 16).map_err(|err| err.to_string())?;
            vec.push(byte);
        }
        Ok(vec)
    }

    //converts a hex string to a hex vector of a fixed number of bytes
    fn to_padded_bytes(&self, len: usize) -> Result<Vec<u8>, String> {
        let mut vec = self.to_bytes().map_err(|err| format!("{}", err))?;
        if vec.len() > len{
            return Err(format!("The length of the hex string must be shorter than the specified maximum of {} characters ({} bytes).", len * 2, len))
        }
        while vec.len() < len {
            vec.insert(0, 0x00);
        }
        Ok(vec)
    }

    //reverses byte order of hex string
    fn reverse_hex (&self) -> Result<String, String>{
        Ok((self.to_bytes()?.reverse()).to_string())
    }
    
    //appends a variable integer to the beginning of string denoting the length of the string
    fn varint(&self) -> String{
        let mut varint: String = varint(self.len()/2).to_string();
        varint + &self
    }

    fn shorten(&self, len: usize) -> String{
        let mut result = String::new();
        for (i, char) in self.chars().enumerate(){
            result.push(char);
            if i == len{
                break;
            }
        }
        result.push_str("...");
        result
    }
}

impl HexString for &str{
    //converts a hex string to hex vector
    fn to_bytes(&self) -> Result<Vec<u8>, String> {
        if self.chars().count() % 2 == 1{
            return Err(format!("Hex strings must have an even number of characters. Your hex string has {} characters.", self.chars().count()))
        }
        let mut vec = Vec::new();
        for i in 0..self.len()/2 {
            let byte = u8::from_str_radix(&self[i*2..i*2+2], 16).map_err(|err| err.to_string())?;
            vec.push(byte);
        }
        Ok(vec)
    }

    //converts a hex string to a hex vector of a fixed number of bytes
    fn to_padded_bytes(&self, len: usize) -> Result<Vec<u8>, String> {
        let mut vec = self.to_bytes().map_err(|err| format!("{}", err))?;
        if vec.len() > len{
            return Err(format!("The length of the hex string must be shorter than the specified maximum of {} characters ({} bytes).", len * 2, len))
        }
        while vec.len() < len {
            vec.insert(0, 0x00);
        }
        Ok(vec)
    }
    
    //reverses byte order of hex string
    fn reverse_hex (&self) -> Result<String, String>{
        Ok((self.to_bytes()?.reverse()).to_string())
    }
    
    //appends a variable integer to the beginning of string denoting the length of the string
    fn varint(&self) -> String{
        let mut varint: String = varint(self.len()/2).to_string();
        varint + &self
    }
    
    fn shorten(&self, len: usize) -> String{
        let mut result = String::new();
        for (i, char) in self.chars().enumerate(){
            result.push(char);
            if i == len{
                break;
            }
        }
        result.push_str("...");
        result
    }
}

pub trait StringHex {
    fn to_string (&self) -> String;
    fn reverse (&self) -> Vec<u8>;
    fn sha256(&self) -> Vec<u8>;
    fn sha256d(&self) -> Vec<u8>;
    fn to_int_le(&self) -> u64;
}

pub trait StringHexArray {
    fn to_string (&self) -> String;
    fn reverse (&mut self);
    fn to_int_le(&self) -> u64;
}

impl StringHex for Vec<u8>{
    //converts a hex vector to hex string
    fn to_string (&self) -> String{
        let mut hex: String = String::new();
        for byte in self {
            hex.push_str(&format!("{:02X}",byte));
        }
        hex
    }
    //reverses the byte order of hex vector
    fn reverse (&self) -> Vec<u8>{
        let mut bytes_reversed: Vec<u8> = Vec::new();
        for byte in self.iter().rev(){
            bytes_reversed.push(*byte);
        }
        return bytes_reversed;
    }
    //creates a sha256 hash of hex string
    fn sha256(&self) -> Vec<u8>{
        let mut hasher = Sha256::default();
        hasher.input(self);
        hasher.result().to_vec()
    }
    //creates a sha256^2 hash of hex string
    fn sha256d(&self) -> Vec<u8>{
        self.sha256().sha256()
    }
    //converts a little endian byte vector to u64 integer equivalent
    fn to_int_le(&self) -> u64{
        assert!(self.len() <= 8, "Byte array is too long to convert to u64");
        let mut result: u64 = 0;
        // let mut power: u64 = 1;
        for (i, byte) in self.iter().enumerate(){
            result |= (*byte as u64) << (i * 8);
        }
        result
    }
}

impl StringHexArray for [u8] {
    //converts a hex array to hex string
    fn to_string(&self) -> String {
        let mut hex: String = String::new();
        for byte in self {
            hex.push_str(&format!("{:02X}", byte));
        }
        hex
    }
    //reverses the byte order of hex array
    fn reverse (&mut self){
        let length = self.len();
        for i in 0..length / 2 {
            let temp = self[i];
            self[i] = self[length - i - 1];
            self[length - i - 1] = temp;
        }
    }
    //converts a little endian byte vector to u64 integer equivalent
    fn to_int_le(&self) -> u64{
        assert!(self.len() <= 8, "Byte array is too long to convert to u64");
        let mut result: u64 = 0;
        // let mut power: u64 = 1;
        for (i, byte) in self.iter().enumerate(){
            result |= (*byte as u64) << (i * 8);
        }
        result
    }
}

//appends a variable integer to the beginning of a byte vector denoting the length of the byte vector
pub fn var (input: Vec<u8>) -> Vec<u8>{
    let input: &mut Vec<u8> = &mut input.clone();
    let mut bytes = varint(input.len());
    bytes.append(input);
    bytes
}

//converts a usize to a variable integer (VarInt / compact size integer) of max 8 bytes
pub fn varint (num: usize) -> Vec<u8>{
    if num < 253{
        return (num as u8).to_le_bytes().to_vec();
    }else{
        let mut b: Vec<u8> = vec![];
        if num < 65536{
            b.push(253);
            b.append(&mut (num as u16).to_le_bytes().to_vec())
        }else if num < 4294967296{
            b.push(254);
            b.append(&mut (num as u32).to_le_bytes().to_vec())
        }else{
            b.push(255);
            b.append(&mut (num as u64).to_le_bytes().to_vec())
        }
        b
    }
}

//converts an amount of coins in satoshi to BTC and returns everything after and including the decimal point
pub fn sat_decimal(n: u64) -> String{
    let n = (n % 100000000) as f64;
    let m = n / 1e8;
    let result = format!("{}",m);
    result.chars().skip(1).collect()
}