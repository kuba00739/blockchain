mod handlers;
pub mod networking;
use crate::networking::send_all;
use bincode::deserialize;
use bincode::serialize;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

const HASH_LEN: usize = 32;

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

impl Vin {
    pub fn new(wmi: Option<String>, vds: Option<String>, vis: Option<String>) -> Vin {
        Vin {
            wmi: wmi.unwrap_or("".to_string()),
            vds: vds.unwrap_or("".to_string()),
            vis: vis.unwrap_or("".to_string()),
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

fn verify_block(block: Block) -> Result<Block, &'static str> {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend(&block.id.to_be_bytes());

    bytes.extend(&block.prev_hash);
    bytes.extend(&serialize(&block.registered_car).unwrap());
    bytes.extend(&serialize(&block.mined_by).unwrap());

    let mut sha2_hash = Sha256::new();
    sha2_hash.update(&bytes);
    sha2_hash.update(block.nonce.to_be_bytes());
    let sum = sha2_hash.finalize();

    if (sum[0] == 0) && (sum[1] == 0) && (sum[2] <= 4) {
        return Ok(block);
    }
    Err("Hash in improper form for this nonce.")
}

fn verify_broadcasted_block(block: Block, blockchain: &Vec<Block>) -> Result<Block, &'static str> {
    debug!("Verifying block: {block}");

    let control_prev_hash: [u8; 32] = if (block.id == 0) || (blockchain.len() == 0) {
        [0; HASH_LEN]
    } else {
        blockchain[((block.id - 1) as usize)].hash
    };

    if control_prev_hash != block.prev_hash {
        return Err("Previous hash don't match!");
    }

    verify_block(block)
}

fn verify_new_block(block: Block, blockchain: &Vec<Block>) -> Result<Block, &'static str> {
    debug!("Verifying block: {block}");

    if (block.id as usize) != blockchain.len() {
        return Err("Block ID don't match blockchain lenght.");
    }

    let control_prev_hash: [u8; 32] = match blockchain.last() {
        Some(s) => s.hash,
        None => [0; HASH_LEN],
    };

    if control_prev_hash != block.prev_hash {
        return Err("Previous hash don't match!");
    }

    verify_block(block)
}

fn mint_block(
    msg: &Msg,
    blockchain: &Vec<Block>,
    blocks_pending: &mut Vec<(Block, u8)>,
    node_name: &String,
) {
    match deserialize::<Car>(&msg.data) {
        Ok(s) => {
            let mut new_block = Block {
                hash: [0; HASH_LEN],
                id: 0,
                nonce: 0,
                prev_hash: [0; HASH_LEN],
                registered_car: s,
                mined_by: node_name.to_string(),
            };

            match blockchain.last() {
                Some(last_block) => new_block.prev_hash = last_block.hash,
                None => new_block.prev_hash = [0; HASH_LEN],
            };

            new_block.id = blockchain.len() as u32;

            let calculated = mine_block(&mut new_block).expect("Error during minting!");
            new_block.nonce = calculated.0;
            new_block.hash = calculated.1;
            blocks_pending.push((new_block.clone(), 1));
            match send_all(Msg {
                command: Comm::NewBlock,
                data: serialize(&new_block).unwrap(),
            }) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Error while multicasting block: {e}");
                }
            }
        }
        Err(e) => {
            warn!("Couldn't deserialize car: {e}");
        }
    }
}

fn mine_block(new_block: &mut Block) -> Result<(u32, [u8; HASH_LEN]), &'static str> {
    let mut bytes: Vec<u8> = Vec::new();

    bytes.extend(&new_block.id.to_be_bytes());
    bytes.extend(&new_block.prev_hash);
    bytes.extend(&serialize(&new_block.registered_car).unwrap());
    bytes.extend(&serialize(&new_block.mined_by).unwrap());

    let mut nonce: u32 = 0;

    while 1 == 1 {
        let mut sha2_hash = Sha256::new();
        sha2_hash.update(&bytes);
        sha2_hash.update(nonce.to_be_bytes());
        let sum = sha2_hash.finalize();
        if (sum[0] == 0) && (sum[1] == 0) && (sum[2] <= 4) {
            let result = match sum.try_into() {
                Err(cause) => panic!("Can't convert a result hash to a slice: {cause}"),
                Ok(result) => result,
            };
            return Ok((nonce, result));
        };
        nonce += 1;
    }
    Err("Nonce couldn't be found")
}

pub fn handle_msg(
    msg: Msg,
    blockchain: &mut Vec<Block>,
    blocks_pending: &mut Vec<(Block, u8)>,
    node_name: &String,
) {
    match msg.command {
        Comm::NewBlock => match handlers::handle_new_block(&msg, blockchain, blocks_pending) {
            Ok(_) => {}
            Err(e) => {
                warn!("Error during new block handling: {e}");
            }
        },
        Comm::Accepted => match handlers::handle_accepted(&msg, blocks_pending) {
            Ok(_) => {}
            Err(e) => {
                warn!("Error while handling accepted: {e}");
            }
        },
        Comm::DataToBlock => {
            mint_block(&msg, blockchain, blocks_pending, node_name);
        }
        Comm::PrintChain => {
            info!("Current blockchain status: \n{:?}", blockchain);
        }
        Comm::Blockchain => match handlers::handle_incoming_blockchain(&msg, &blockchain) {
            Ok(s) => {
                info!("Accepting new blockchain");
                *blockchain = s;
            }
            Err(e) => {
                debug!("New blockchain verification failed: {e}");
            }
        },

        _ => {}
    }
}
