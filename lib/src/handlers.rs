use log::info;
use log::{debug, warn};

use crate::datatypes::{BlockData, BlockchainError, ContractResult};
use crate::verify_new_block;
use crate::Block;
use crate::Comm;
use crate::Msg;
use crate::{ret_err, send_all};
use crate::{reverse_polish, verify_broadcasted_block};
use bincode::deserialize;
use bincode::serialize;
use crossbeam_channel::Sender;

pub fn handle_new_block(
    msg: &Msg,
    blockchain: &mut Vec<Block>,
    tx: &Sender<Msg>,
) -> Result<(), Box<dyn std::error::Error>> {
    let block = deserialize::<Block>(&msg.data)?;
    if (block.id as usize) != blockchain.len() {
        debug!("Block ID didn't match!");
        return Ok(());
    };
    verify_new_block(block.clone(), blockchain)?;
    match send_all(Msg {
        command: Comm::Accepted,
        data: serialize(&block).unwrap(),
    }) {
        Ok(_) => {}
        Err(e) => {
            warn!("Error while multicasting accepted message: {e}");
        }
    }

    tx.send(Msg {
        command: Comm::EndMining,
        data: Vec::new(),
    })?;
    info!("Adding new block: {block}");
    blockchain.push(block);
    Ok(())
}

pub fn handle_incoming_blockchain(
    msg: &Msg,
    current_blockchain: &Vec<Block>,
) -> Result<Vec<Block>, Box<dyn std::error::Error>> {
    let new_blockchain = deserialize::<Vec<Block>>(&msg.data)?;
    if current_blockchain.len() >= new_blockchain.len() {
        ret_err!("New block is shorter or equal in lenght to current one.");
    }
    let mut ctr: u32 = 0;
    for block in &new_blockchain {
        if block.id != ctr {
            ret_err!("Block id incorrect");
        }
        verify_broadcasted_block(block.clone(), &new_blockchain.as_ref())?;
        ctr += 1;
    }
    return Ok(new_blockchain.clone());
}

pub fn handle_calc_contract(
    msg: &Msg,
    tx: &std::sync::mpsc::Sender<Msg>,
    blockchain: &mut Vec<Block>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = deserialize::<Vec<f64>>(&msg.data)?;
    let block_id: usize;
    match args.pop() {
        Some(s) => {
            block_id = s as usize;
        }
        None => {
            ret_err!("Couldn't extract block id");
        }
    }
    if block_id >= blockchain.len() {
        ret_err!("Block id is bigger than blockchain lenght");
    }
    let block_data = &blockchain[block_id].data;

    match block_data {
        crate::BlockData::Contract(s) => {
            let data = BlockData::ContractResult(ContractResult {
                block_id: (block_id as u32),
                result: reverse_polish(s, &args)?,
                args,
            });
            tx.send(Msg {
                command: Comm::DataToBlock,
                data: serialize(&data)?,
            })?;
        }
        _ => {
            ret_err!("Provided block doesn't hold contract.");
        }
    }

    Ok(())
}
