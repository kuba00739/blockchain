use log::{debug, warn};

use crate::send_all;
use crate::verify_broadcasted_block;
use crate::verify_new_block;
use crate::Block;
use crate::Comm;
use crate::Msg;
use bincode::deserialize;
use bincode::serialize;

pub fn handle_new_block(
    msg: &Msg,
    blockchain: &Vec<Block>,
    nodes: &Vec<&str>,
    block_pending: &mut Vec<(Block, u8)>,
) {
    match deserialize::<Block>(&msg.data) {
        Ok(s) => {
            if (s.id as usize) != blockchain.len() {
                debug!("Block ID didn't match!");
                return;
            }
            match verify_new_block(s, blockchain) {
                Ok(s) => {
                    send_all(
                        Msg {
                            command: Comm::Accepted,
                            data: serialize(&s).unwrap(),
                        },
                        nodes,
                    );

                    block_pending.push((s, 1));
                }
                Err(e) => {
                    debug!("Verification failed: {e}");
                }
            }
        }
        Err(e) => {
            warn!("Error while deserializing {e}");
        }
    }
}

pub fn handle_accepted(msg: &Msg, blocks_pending: &mut Vec<(Block, u8)>) {
    match deserialize::<Block>(&msg.data) {
        Ok(s) => {
            for pending in blocks_pending {
                if s == pending.0 {
                    pending.1 += 1;
                }
            }
        }
        Err(e) => {
            warn!("Couldn't deserialize block: {e}");
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
                return Err("New block is shorter or equal in lenght to current one.");
            }
            let mut ctr: u32 = 0;
            for block in &s {
                if block.id != ctr {
                    return Err("Block id incorrect");
                }
                match verify_broadcasted_block(block.clone(), s.as_ref()) {
                    Ok(_) => {}
                    Err(_) => {
                        return Err("Blockchain verification failed.");
                    }
                }
                ctr += 1;
            }
            return Ok(s.clone());
        }
        Err(_) => {
            return Err("Error while deserializing blockchain.");
        }
    }
}
