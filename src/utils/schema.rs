use {
    crate::utils::{
        env_var::get_env_var, get_block::get_current_block_number,
        planetscale::ps_get_archived_blocks_count, transaction::get_balance_of,
    },
    borsh::{from_slice, to_vec},
    borsh_derive::{BorshDeserialize, BorshSerialize},
    ethers::types::U256,
    ethers_providers::{Http, Provider},
    planetscale_driver::Database,
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::{
        collections::HashMap,
        convert::TryFrom,
        fs::File,
        io::{Read, Write},
    },
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Network {
    pub name: String,
    pub wvm_chain_id: u32,
    pub network_rpc: String,
    pub wvm_rpc: String,
    pub block_time: f32,
    pub start_block: u64, // as per ethers_provider
    pub archiver_address: String,
    pub backfill_address: String,
    pub archive_pool_address: String,
}

impl Network {
    pub fn config() -> Network {
        let network_config = get_env_var("network").unwrap();
        let mut file = File::open(network_config).unwrap();
        let mut data = String::new();

        file.read_to_string(&mut data).unwrap();

        let network: Network = serde_json::from_str(&data).unwrap();
        // cannot self send data
        assert_ne!(network.archiver_address, network.archive_pool_address);
        network
    }

    pub async fn provider(&self, rpc: bool) -> Provider<Http> {
        let target_rpc: &String;

        let network: Network = Self::config();
        if rpc {
            target_rpc = &network.wvm_rpc;
        } else {
            target_rpc = &network.network_rpc
        }
        let provider: Provider<Http> =
            Provider::<Http>::try_from(target_rpc).expect("could not instantiate HTTP Provider");

        provider
    }
}

#[derive(Debug, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Block {
    pub number: String,
    pub hash: String,
    #[serde(rename = "parentHash")]
    pub parent_hash: String,
    #[serde(rename = "stateRoot")]
    pub state_root: String,
    #[serde(rename = "extrinsicsRoot")]
    pub extrinsics_root: String,
    #[serde(rename = "authorId")]
    pub author_id: String,
    pub logs: Vec<Log>,
    #[serde(rename = "onInitialize")]
    pub on_initialize: InitializeFinalize,
    pub extrinsics: Vec<Extrinsic>,
    #[serde(rename = "onFinalize")]
    pub on_finalize: InitializeFinalize,
    pub finalized: bool,
}

#[derive(Debug, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Log {
    #[serde(rename = "type")]
    pub log_type: String,
    pub index: String,
    pub value: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct InitializeFinalize {
    pub events: Vec<Event>,
}

#[derive(Debug, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Extrinsic {
    pub method: Method,
    pub signature: Option<String>,
    pub nonce: Option<String>,
    pub args: String, // Just store as string
    pub tip: Option<String>,
    pub hash: String,
    pub info: String, // Just store as string
    pub era: Era,
    pub events: Vec<Event>,
    pub success: bool,
    #[serde(rename = "paysFee")]
    pub pays_fee: bool,
}

#[derive(Debug, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct JsonMap {
    #[serde(flatten)]
    pub data: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Method {
    pub pallet: String,
    pub method: String,
}

#[derive(Debug, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Era {
    #[serde(rename = "immortalEra")]
    pub immortal_era: String,
}

#[derive(Debug, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct EventData {
    pub weight: Weight, // Not optional anymore
    pub class: String,
    #[serde(rename = "paysFee")]
    pub pays_fee: String,
}

#[derive(Debug, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq)]
#[serde(untagged)]
pub enum DataValue {
    EventData(EventData),
    Str(String),
    Array(Vec<String>),
    Numeric(u64), // If there's a chance for numeric values
}

#[derive(Debug, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Event {
    pub method: Method,
    #[serde(default)]
    pub data: Vec<DataValue>,
}

#[derive(Debug, Deserialize, Serialize, BorshSerialize, BorshDeserialize, PartialEq)]
pub struct Weight {
    #[serde(rename = "refTime")]
    pub ref_time: Option<String>,
    #[serde(rename = "proofSize")]
    pub proof_size: Option<String>,
}

#[derive(Database, Debug, Serialize)]
pub struct PsGetBlockTxid {
    pub wvm_archive_txid: String,
}

#[derive(Database, Debug, Serialize)]
pub struct PsGetExtremeBlock {
    pub block_id: u64,
}

#[derive(Database, Debug, Serialize)]
pub struct PsGetTotalBlocksCount {
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct InfoServerResponse {
    first_livesync_archived_block: Option<u64>,
    last_livesync_archived_block: Option<u64>,
    first_backfill_archived_block: Option<u64>,
    last_backfill_archived_block: Option<u64>,
    livesync_start_block: u64,
    total_archived_blocks: u64,
    blocks_behind_live_blockheight: u64,
    archiver_balance: U256,
    archiver_address: String,
    backfill_address: String,
    backfill_balance: U256,
    network_name: String,
    network_rpc: String,
}

impl InfoServerResponse {
    pub async fn new(
        first_livesync_block: Option<u64>,
        last_livesync_block: Option<u64>,
        first_backfill_block: Option<u64>,
        last_backfill_block: Option<u64>,
    ) -> InfoServerResponse {
        let network = Network::config();
        // balances
        let archiver_balance = get_balance_of(network.archiver_address.clone()).await;
        let archiver_balance = Some(archiver_balance).unwrap_or("0".into());
        let backfill_balance = get_balance_of(network.backfill_address.clone()).await;
        let backfill_balance = Some(backfill_balance).unwrap_or("0".into());
        // blocks stats
        let total_archived_blocks = ps_get_archived_blocks_count().await;
        let current_live_block = get_current_block_number().await;
        let blocks_behind_live_blockheight = current_live_block - last_livesync_block.unwrap_or(0);

        let instance: InfoServerResponse = InfoServerResponse {
            archiver_balance,
            backfill_balance,
            blocks_behind_live_blockheight,
            livesync_start_block: network.start_block,
            first_livesync_archived_block: first_livesync_block,
            first_backfill_archived_block: first_backfill_block,
            last_livesync_archived_block: last_livesync_block,
            last_backfill_archived_block: last_backfill_block,
            total_archived_blocks,
            archiver_address: network.archiver_address,
            backfill_address: network.backfill_address,
            network_name: network.name,
            network_rpc: network.network_rpc,
        };
        instance
    }
}

impl Block {
    pub fn load_block_from_value(mut value: Value) -> Result<Block, serde_json::Error> {
        if let Some(extrinsics) = value.get_mut("extrinsics").and_then(|e| e.as_array_mut()) {
            for extrinsic in extrinsics {
                if let Some(obj) = extrinsic.as_object_mut() {
                    if let Some(args) = obj.get("args") {
                        obj.insert(
                            "args".to_string(),
                            Value::String(serde_json::to_string(args)?),
                        );
                    }
                    if let Some(info) = obj.get("info") {
                        obj.insert(
                            "info".to_string(),
                            Value::String(serde_json::to_string(info)?),
                        );
                    }
                    if let Some(events) = obj.get_mut("events").and_then(|e| e.as_array_mut()) {
                        for event in events {
                            if let Some(data) = event.get_mut("data") {
                                // Normalize data into Borsh-compatible formats
                                match data {
                                    Value::Array(arr) => {
                                        let normalized: Vec<String> = arr
                                            .iter()
                                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                            .collect();
                                        *data = serde_json::to_value(normalized).unwrap();
                                    }
                                    Value::Object(_) => {
                                        // Convert objects to a string for now
                                        *data = Value::String(serde_json::to_string(data)?);
                                    }
                                    _ => {} // Other formats are already compatible
                                }
                            }
                        }
                    }
                }
            }
        }
        serde_json::from_value(value)
    }
}

impl Block {
    pub fn brotli_compress(input: &[u8]) -> Vec<u8> {
        let mut writer = brotli::CompressorWriter::new(Vec::new(), 4096, 11, 22);
        writer.write_all(input).unwrap();
        writer.into_inner()
    }
    pub fn brotli_decompress(input: Vec<u8>) -> Vec<u8> {
        let mut decompressed_data = Vec::new();
        let mut decompressor = brotli::Decompressor::new(input.as_slice(), 4096); // 4096 is the buffer size

        decompressor
            .read_to_end(&mut decompressed_data)
            .expect("Decompression failed");
        decompressed_data
    }
    pub fn borsh_ser(input: &Block) -> Vec<u8> {
        to_vec(input).unwrap()
    }
    pub fn borsh_der(input: Vec<u8>) -> Block {
        let res: Block = from_slice(&input).expect("error deseriliazing the calldata");
        res
    }
}