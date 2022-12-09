use serde::{Deserialize, Serialize};
use std::fmt;

pub const HASH_LEN: usize = 32;

#[derive(Debug)]
pub struct BlockchainError(pub String);

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Vin {
    wmi: String,
    vds: String,
    vis: String,
}
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Car {
    owner_name: String,
    owner_surname: String,
    distance_traveled: u32,
    vin_number: Vin,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct Block {
    pub hash: [u8; HASH_LEN],
    pub id: u32,
    pub prev_hash: [u8; HASH_LEN],
    pub nonce: u32,
    pub data: BlockData,
    pub mined_by: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
pub enum RevPolish {
    Number(i32),
    Operation(char),
    Arg,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub struct ContractResult {
    pub block_id: u32,
    pub result: u32,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum BlockData {
    Contract(Vec<RevPolish>),
    Car(Car),
    ContractResult(ContractResult),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Comm {
    NewBlock,
    Accepted,
    Rejected,
    DataToBlock,
    PrintChain,
    Broadcast,
    Blockchain,
    EndMining,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct Msg {
    pub command: Comm,
    pub data: Vec<u8>,
}

impl Car {
    pub fn new(
        owner_name: Option<String>,
        owner_surname: Option<String>,
        distance_traveled: Option<u32>,
        vin_number: Option<Vin>,
    ) -> Car {
        Car {
            owner_name: owner_name.unwrap_or("".to_string()),
            owner_surname: owner_surname.unwrap_or("".to_string()),
            distance_traveled: distance_traveled.unwrap_or(0),
            vin_number: vin_number.unwrap_or(Vin::new(None, None, None)),
        }
    }
}

impl Block {
    pub fn new_empty() -> Block {
        Block {
            hash: [0; HASH_LEN],
            id: 0,
            prev_hash: [0; HASH_LEN],
            nonce: 0,
            data: BlockData::Car(Car::new(None, None, None, None)),
            mined_by: "".to_string(),
        }
    }
}

impl Vin {
    pub fn new(wmi: Option<String>, vds: Option<String>, vis: Option<String>) -> Vin {
        Vin {
            wmi: wmi.unwrap_or("".to_string()),
            vds: vds.unwrap_or("".to_string()),
            vis: vis.unwrap_or("".to_string()),
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Block [ID: {} Hash: {} Prev Hash: {} Miner: {} Data: {}]\n",
            self.id,
            format_hash(self.hash),
            format_hash(self.prev_hash),
            self.mined_by,
            self.data,
        )
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Block [ID: {} Hash: {} Prev Hash: {} Miner: {} Nonce: {}]\n",
            self.id,
            format_hash(self.hash),
            format_hash(self.prev_hash),
            self.mined_by,
            self.nonce
        )
    }
}

impl fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.0)
    }
}

impl std::error::Error for BlockchainError {}

impl fmt::Display for BlockData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockData::Contract(s) => {
                write!(f, "Contract: {:?}", s)
            }
            BlockData::Car(s) => {
                write!(f, "Car owner: {} {}", s.owner_name, s.owner_surname)
            }
            BlockData::ContractResult(s) => {
                write!(f, "Contract ID: {}, result: {}", s.block_id, s.result)
            }
        }
    }
}

fn format_hash(hash: [u8; HASH_LEN]) -> String {
    let mut formatted = String::new();
    for i in &hash[0..8] {
        formatted += &format!("{:2x}", i).to_string();
    }
    formatted += &"...".to_string();
    formatted
}
