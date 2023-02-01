#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Vin {
    pub txid: String,
    pub vout: Option<u32>,
    pub sequence: Option<u32>,
    pub n: u32,
    pub addresses: Vec<String>,
    pub is_address: bool,
    pub value: String,
    // pub hex: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Vout {
    pub value: String,
    pub n: u32,
    pub spent: Option<bool>,
    pub hex: String,
    pub addresses: Vec<String>,
    pub is_address: bool
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainTransaction {
    pub txid: String,
    pub version: u32,
    // pub lock_time: u64,
    pub vin: Vec<Vin>,
    pub vout: Vec<Vout>,
    pub block_hash: Option<String>,
    pub block_height: i64,
    pub confirmations: u64,
    pub block_time: u64,
    pub size: u64,
    pub vsize: u64,
    pub value: String,
    pub value_in: String,
    pub fees: String,
    pub hex: String,
}