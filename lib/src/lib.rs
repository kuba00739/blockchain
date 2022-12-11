pub mod datatypes;
mod handlers;
pub mod networking;
pub use crate::datatypes::{Block, BlockData, Car, Comm, Msg, RevPolish, HASH_LEN};
use crate::networking::{broadcast_chain, send_all};
use bincode::deserialize;
use bincode::serialize;
use crossbeam_channel::unbounded;
use crossbeam_channel::{Receiver, Sender};
use datatypes::BlockchainError;
use datatypes::RevPolish::{Arg, Number, Operation};
use handlers::handle_calc_contract;
use log::{debug, info, warn};
use sha2::{Digest, Sha256};
use std::sync::mpsc::Sender as StdSender;
use std::thread;
use std::thread::JoinHandle;
#[macro_export]
macro_rules! ret_err {
    ( $x:expr ) => {{
        return Err(Box::new(BlockchainError($x.into())));
    }};
}

fn verify_block(block: Block) -> Result<Block, Box<dyn std::error::Error>> {
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend(&block.id.to_be_bytes());

    bytes.extend(&block.prev_hash);
    bytes.extend(&serialize(&block.data).unwrap());
    bytes.extend(&serialize(&block.mined_by).unwrap());

    let mut sha2_hash = Sha256::new();
    sha2_hash.update(&bytes);
    sha2_hash.update(block.nonce.to_be_bytes());
    let sum = sha2_hash.finalize();

    if (sum[0] == 0) && (sum[1] == 0) && (sum[2] == 0) && (sum[3] <= 128) {
        return Ok(block);
    }
    ret_err!("Hash in improper form for this nonce.");
}

fn verify_broadcasted_block(
    block: Block,
    blockchain: &Vec<Block>,
) -> Result<Block, Box<dyn std::error::Error>> {
    debug!("Verifying block: {block}");

    let control_prev_hash: [u8; 32] = if (block.id == 0) || (blockchain.len() == 0) {
        [0; HASH_LEN]
    } else {
        blockchain[((block.id - 1) as usize)].hash
    };

    if control_prev_hash != block.prev_hash {
        ret_err!("Previous hash don't match!");
    }

    verify_block(block)
}

fn verify_new_block(
    block: Block,
    blockchain: &Vec<Block>,
) -> Result<Block, Box<dyn std::error::Error>> {
    debug!("Verifying block: {block}");

    if (block.id as usize) != blockchain.len() {
        ret_err!("Block ID don't match blockchain lenght.");
    }

    let control_prev_hash: [u8; 32] = match blockchain.last() {
        Some(s) => s.hash,
        None => [0; HASH_LEN],
    };

    if control_prev_hash != block.prev_hash {
        ret_err!("Previous hash don't match!");
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
    let mut new_block = Block {
        hash: [0; HASH_LEN],
        id: 0,
        nonce: 0,
        prev_hash: [0; HASH_LEN],
        data: deserialize::<BlockData>(&msg.data)?,
        mined_by: node_name.to_string(),
    };

    new_block.prev_hash = last_block.hash;
    new_block.id = if new_block.prev_hash == [0; HASH_LEN] {
        0
    } else {
        last_block.id + 1
    };

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
) -> Result<(u32, [u8; HASH_LEN]), Box<dyn std::error::Error>> {
    let mut bytes: Vec<u8> = Vec::new();

    bytes.extend(&new_block.id.to_be_bytes());
    bytes.extend(&new_block.prev_hash);
    bytes.extend(&serialize(&new_block.data)?);
    bytes.extend(&serialize(&new_block.mined_by)?);

    let mut nonce: u32 = 0;

    while rx.is_empty() {
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
    ret_err!("Mining stopped via message.");
}

fn start_miner_thread(
    msg: Msg,
    blocks: &Vec<Block>,
    node_name: &String,
    tx_mpsc: &std::sync::mpsc::Sender<Msg>,
    rx_mpmc: &Receiver<Msg>,
) -> Option<JoinHandle<()>> {
    let last_block = match blocks.last() {
        Some(s) => s.clone(),
        None => Block::new_empty().clone(),
    };
    let node_name_clone = node_name.clone();
    let tx_mpsc_clone = tx_mpsc.clone();
    let rx_mpmc_clone = rx_mpmc.clone();

    let miner_thread = Some(thread::spawn({
        move || match mint_block(
            &msg,
            last_block,
            &node_name_clone,
            tx_mpsc_clone,
            rx_mpmc_clone,
        ) {
            Ok(_) => {}
            Err(e) => {
                debug!("Error during minting: {e}");
            }
        }
    }));
    return miner_thread;
}

pub fn handle_msg(
    msg: Msg,
    blockchain: &mut Vec<Block>,
    is_miner_running: &mut bool,
    miner_thread: &mut Option<JoinHandle<()>>,
    node_name: &String,

    tx_mpsc: &std::sync::mpsc::Sender<Msg>,
    tx_mpmc: &mut Sender<Msg>,
    rx_mpmc: &mut Receiver<Msg>,
) {
    match msg.command {
        Comm::DataToBlock => {
            if *is_miner_running {
                *is_miner_running = !(miner_thread.as_ref().unwrap().is_finished());
                if *is_miner_running {
                    return;
                }
                (*tx_mpmc, *rx_mpmc) = unbounded::<Msg>();
            }

            *miner_thread = start_miner_thread(msg, blockchain, &node_name, &tx_mpsc, &rx_mpmc);

            *is_miner_running = true;
        }
        Comm::Broadcast => {
            broadcast_chain(blockchain);
        }

        Comm::NewBlock => {
            if let Err(e) = handlers::handle_new_block(&msg, blockchain, &tx_mpmc) {
                warn!("Error during new block handling: {e}");
            }
        }
        Comm::PrintChain => {
            info!("Current blockchain status: \n{:#?}", blockchain);
        }
        Comm::Blockchain => match handlers::handle_incoming_blockchain(&msg, &blockchain) {
            Ok(s) => {
                info!("Accepting new blockchain");
                match tx_mpmc.send(Msg {
                    command: Comm::EndMining,
                    data: Vec::new(),
                }) {
                    Ok(_) => {}
                    Err(e) => {
                        warn!("Error sending message to miner thread: {e}");
                    }
                }
                *blockchain = s;
            }
            Err(e) => {
                debug!("New blockchain verification failed: {e}");
            }
        },
        Comm::CalcContract => match handle_calc_contract(&msg, tx_mpsc, blockchain) {
            Ok(()) => {
                info!("Calculated contract value");
            }
            Err(e) => {
                warn!("Error calculating contract: {e}");
            }
        },

        _ => {}
    }
}

fn reverse_polish(
    contract_orig: &Vec<RevPolish>,
    args_orig: &Vec<f64>,
) -> Result<f64, Box<dyn std::error::Error>> {
    let mut parsed_ints: Vec<f64> = Vec::new();
    let mut contract = contract_orig.clone();
    let mut args = args_orig.clone();

    loop {
        let value: RevPolish;
        match contract.pop() {
            Some(s) => {
                value = s;
            }
            None => {
                return Ok(parsed_ints.pop().unwrap());
            }
        }
        match value {
            Number(n) => {
                parsed_ints.push(n);
            }
            Operation(c) => {
                let result: f64;
                match c {
                    '+' => {
                        result = parsed_ints
                            .pop()
                            .ok_or(BlockchainError("No value found".to_string()))?
                            + parsed_ints
                                .pop()
                                .ok_or(BlockchainError("No value found".to_string()))?;
                        parsed_ints.push(result);
                    }
                    '-' => {
                        result = parsed_ints
                            .pop()
                            .ok_or(BlockchainError("No value found".to_string()))?
                            - parsed_ints
                                .pop()
                                .ok_or(BlockchainError("No value found".to_string()))?;
                        parsed_ints.push(result);
                    }
                    '*' => {
                        result = parsed_ints
                            .pop()
                            .ok_or(BlockchainError("No value found".to_string()))?
                            * parsed_ints
                                .pop()
                                .ok_or(BlockchainError("No value found".to_string()))?;
                        parsed_ints.push(result);
                    }
                    '%' => {
                        let val1 = parsed_ints
                            .pop()
                            .ok_or(BlockchainError("No value found".to_string()))?;
                        let val2 = parsed_ints
                            .pop()
                            .ok_or(BlockchainError("No value found".to_string()))?;

                        if val2 == 0.0 {
                            ret_err!("Error: division by 0");
                        }

                        parsed_ints.push(val1 % val2);
                    }
                    '/' => {
                        let val1 = parsed_ints
                            .pop()
                            .ok_or(BlockchainError("No value found".to_string()))?;
                        let val2 = parsed_ints
                            .pop()
                            .ok_or(BlockchainError("No value found".to_string()))?;

                        if val2 == 0.0 {
                            ret_err!("Error: division by 0");
                        }

                        parsed_ints.push(val1 / val2);
                    }
                    'p' => {
                        let val1 = parsed_ints
                            .pop()
                            .ok_or(BlockchainError("No value found".to_string()))?;
                        let val2 = parsed_ints
                            .pop()
                            .ok_or(BlockchainError("No value found".to_string()))?;

                        parsed_ints.push(val1.powf(val2));
                    }

                    _ => {
                        ret_err!("Unknown sign");
                    }
                }
            }
            Arg => {
                parsed_ints.push(
                    args.pop()
                        .ok_or(BlockchainError("No value found".to_string()))?,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::vec;

    use crate::{
        datatypes::RevPolish::Arg, datatypes::RevPolish::Number, datatypes::RevPolish::Operation,
        reverse_polish,
    };

    #[test]
    fn test_rev_polish() {
        let mut input = vec![Operation('+'), Number(0.0), Number(1.0)];
        assert_eq!(reverse_polish(&mut input, &mut Vec::new()).unwrap(), 1.0);
        input = vec![
            Operation('*'),
            Number(2.0),
            Operation('+'),
            Number(3.0),
            Number(5.0),
        ];
        assert_eq!(reverse_polish(&mut input, &mut Vec::new()).unwrap(), 16.0);
        input = vec![
            Operation('*'),
            Number(2.0),
            Operation('-'),
            Number(3.0),
            Number(5.0),
        ];
        assert_eq!(reverse_polish(&mut input, &mut Vec::new()).unwrap(), -4.0);

        let mut args: Vec<f64> = vec![5.0, 3.0];
        input = vec![Operation('*'), Number(2.0), Operation('-'), Arg, Arg];

        assert_eq!(reverse_polish(&mut input, &mut args).unwrap(), -4.0);
    }
}
