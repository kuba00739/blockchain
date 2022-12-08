use serde::{Deserialize, Serialize};
use std::fmt;

pub const HASH_LEN: usize = 32;

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
    pub registered_car: Car,
    pub mined_by: String,
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
            registered_car: Car::new(None, None, None, None),
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
            "Block [ID: {} Hash: {} Prev Hash: {} Miner: {} Car owner: {} {}]\n",
            self.id,
            format_hash(self.hash),
            format_hash(self.prev_hash),
            self.mined_by,
            self.registered_car.owner_name,
            self.registered_car.owner_surname
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

fn format_hash(hash: [u8; HASH_LEN]) -> String {
    let mut formatted = String::new();
    for i in &hash[0..8] {
        formatted += &format!("{:2x}", i).to_string();
    }
    formatted += &"...".to_string();
    formatted
}
