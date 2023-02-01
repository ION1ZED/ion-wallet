#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UTXO {
    pub txid: String,
    pub vout: u32,
    pub value: String,
    pub height: Option<u64>,
    pub confirmations: u64
}