use crate::send_all;
use crate::Msg;
use crate::Block;
use crate::verify_new_block;
use crate::Comm;
use crate::verify_broadcasted_block;
use bincode::deserialize;
use bincode::serialize;
use serde::{Deserialize, Serialize};

pub fn handle_new_block(
    msg: &Msg,
    blockchain: &Vec<Block>,
    nodes: &Vec<&str>,
    block_pending: &mut (Block, u8),
) {
    match deserialize::<Block>(&msg.data) {
        Ok(s) => match verify_new_block(s, blockchain) {
            Ok(s) => {
                send_all(
                    Msg {
                        command: Comm::Accepted,
                        data: serialize(&s).unwrap(),
                    },
                    nodes,
                );

                *block_pending = (s, 1);
            }
            Err(e) => {
                eprintln!("Verification failed: {e}");
            }
        },
        Err(e) => {
            eprintln!("Error while deserializing {e}");
        }
    }
}

pub fn handle_accepted(msg: &Msg, block_pending: &mut (Block, u8)) {
    match deserialize::<Block>(&msg.data) {
        Ok(s) => {
            if s == block_pending.0 {
                block_pending.1 += 1;
            }
        }
        Err(e) => {
            eprintln!("Couldn't deserialize block: {e}");
        }
    }
}

pub fn handle_incoming_blockchain(
    msg: &Msg,
    current_blockchain: &Vec<Block>,
) -> Result<Vec<Block>, &'static str> {
    match deserialize::<Vec<Block>>(&msg.data) {
        Ok(s) => {
            if current_blockchain.len() >= s.len() {
                eprintln!("New block is shorter or equal in lenght to current one.");
                return Err("New block is shorter or equal in lenght to current one.");
            }
            let mut ctr: u32 = 0;
            for block in &s {
                if block.id != ctr {
                    eprintln!("Block id incorrect");
                    return Err("Block id incorrect");
                }
                match verify_broadcasted_block(block.clone(), s.as_ref()) {
                    Ok(_) => {}
                    Err(_) => {
                        eprintln!("Blockchain verification failed.");
                        return Err("Blockchain verification failed.");
                    }
                }
                ctr += 1;
            }
            return Ok(s.clone());
        }
        Err(_) => {
            eprintln!("Error while deserializing blockchain.");
            return Err("Error while deserializing blockchain.");
        }
    }
}