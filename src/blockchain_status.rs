#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlockchainStatus {
    pub blockbook: Blockbook,
    pub backend: Backend,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Blockbook {
    pub coin: String,
    pub host: String,
    pub version: String,
    pub git_commit: String,
    pub build_time: String,
    pub sync_mode: bool,
    pub initial_sync: bool,
    pub in_sync: bool,
    pub best_height: u64,
    pub last_block_time: String,
    pub in_sync_mempool: bool,
    pub last_mempool_time: String,
    pub mempool_size: u64,
    pub decimals: u64,
    pub db_size: u64,
    pub about: String
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Backend {
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    pub best_block_hash: String,
    pub difficulty: String,
    pub size_on_disk: u64,
    pub version: String,
    pub subversion: String,
    pub protocol_version: String
}