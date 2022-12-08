pub mod datatypes;
mod handlers;
pub mod networking;
pub use crate::datatypes::{Block, Car, Comm, Msg, HASH_LEN};
use crate::networking::send_all;
use bincode::deserialize;
use bincode::serialize;
use crossbeam_channel::{Receiver, Sender};
use log::{debug, info, warn};
use sha2::{Digest, Sha256};
use std::sync::mpsc::Sender as StdSender;

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

    if (sum[0] == 0) && (sum[1] == 0) && (sum[2] == 0) && (sum[3] <= 128) {
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

pub fn mint_block(
    msg: &Msg,
    last_block: Block,
    node_name: &String,
    tx: StdSender<Msg>,
    rx: Receiver<Msg>,
) -> Result<(), Box<dyn std::error::Error>> {
    let car = deserialize::<Car>(&msg.data)?;
    let mut new_block = Block {
        hash: [0; HASH_LEN],
        id: 0,
        nonce: 0,
        prev_hash: [0; HASH_LEN],
        registered_car: car,
        mined_by: node_name.to_string(),
    };

    new_block.prev_hash = last_block.hash;
    if new_block.prev_hash == [0; HASH_LEN] {
        new_block.id = 0;
    } else {
        new_block.id = last_block.id + 1;
    }

    let calculated = mine_block(&mut new_block, rx)?;
    new_block.nonce = calculated.0;
    new_block.hash = calculated.1;

    tx.send(Msg {
        command: Comm::NewBlock,
        data: serialize(&new_block)?,
    })?;

    send_all(Msg {
        command: Comm::NewBlock,
        data: serialize(&new_block).unwrap(),
    })?;
    Ok(())
}

fn mine_block(
    new_block: &mut Block,
    rx: Receiver<Msg>,
) -> Result<(u32, [u8; HASH_LEN]), &'static str> {
    let mut bytes: Vec<u8> = Vec::new();

    bytes.extend(&new_block.id.to_be_bytes());
    bytes.extend(&new_block.prev_hash);
    bytes.extend(&serialize(&new_block.registered_car).unwrap());
    bytes.extend(&serialize(&new_block.mined_by).unwrap());

    let mut nonce: u32 = 0;

    while 1 == 1 {
        if !rx.is_empty() {
            return Err("Mining stopped via message.");
        }
        let mut sha2_hash = Sha256::new();
        sha2_hash.update(&bytes);
        sha2_hash.update(nonce.to_be_bytes());
        let sum = sha2_hash.finalize();
        if (sum[0] == 0) && (sum[1] == 0) && (sum[2] == 0) && (sum[3] <= 128) {
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

pub fn handle_msg(msg: Msg, blockchain: &mut Vec<Block>, tx: &Sender<Msg>) {
    match msg.command {
        Comm::NewBlock => match handlers::handle_new_block(&msg, blockchain, tx) {
            Ok(_) => {}
            Err(e) => {
                warn!("Error during new block handling: {e}");
            }
        },
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
