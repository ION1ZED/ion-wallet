#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainAddress {
    pub page: u64,
    pub total_pages: u64,
    pub items_on_page: u64,
    pub address: String,
    pub balance: String,
    pub total_received: String,
    pub total_sent: String,
    pub unconfirmed_balance: String,
    pub unconfirmed_txs: u64,
    pub txs: u64,
    pub txids: Vec<String>
}