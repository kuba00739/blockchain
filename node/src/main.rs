use chrono::Local;
use crossbeam_channel::unbounded;
use env_logger::Builder;
use lib::mint_block;
use lib::{handle_msg, networking::broadcast_chain, networking::listen};
use lib::{Block, Comm, Msg};
use log::{debug, info};
use log::{warn, LevelFilter};
use std::env;
use std::io::Write;
use std::sync::mpsc;
use std::thread::sleep;
use std::thread::{self, JoinHandle};
use std::time::Duration;

//TODO- new thread for block mining: Can use handle_new_block

fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let (tx_listener, rx_main) = mpsc::channel::<Msg>();
    let (mut tx_main_mint, mut rx_main_mint) = unbounded::<Msg>();

    let node_name = env::var("NAME").expect("Couldn't access NODES env variable.");

    let mut blocks: Vec<Block> = Vec::new();
    let mut blocks_pending: Vec<(Block, u8)> = Vec::new();

    let tx1 = tx_listener.clone();

    thread::spawn({
        move || {
            listen(tx1);
        }
    });

    let tx2 = tx_listener.clone();

    thread::spawn({
        move || loop {
            sleep(Duration::new(60, 0));
            tx2.send(Msg {
                command: lib::Comm::Broadcast,
                data: Vec::new(),
            })
            .expect("Message to main thread couldn't be sent.");
        }
    });

    let mut miner_thread: Option<JoinHandle<()>> = None;
    let mut is_miner_running: bool = false;

    for msg in rx_main {
        debug!("Received msg: {:#?}", msg);
        match msg.command {
            Comm::Broadcast => {
                broadcast_chain(&blocks);
            }
            Comm::DataToBlock => {
                match is_miner_running {
                    true => {
                        is_miner_running = !(miner_thread.as_ref().unwrap().is_finished());
                        if is_miner_running {
                            continue;
                        }
                    }
                    false => {}
                }

                let last_block = match blocks.last() {
                    Some(s) => s.clone(),
                    None => Block::new_empty().clone(),
                };
                let node_name_clone = node_name.clone();
                let tx_node = tx_listener.clone();
                let rx_main_mint_clone = rx_main_mint.clone();

                miner_thread = Some(thread::spawn({
                    move || match mint_block(
                        &msg,
                        last_block,
                        &node_name_clone,
                        tx_node,
                        rx_main_mint_clone,
                    ) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Error during minting: {e}");
                        }
                    }
                }));
                is_miner_running = true;
                continue;
            }
            _ => {}
        }
        handle_msg(msg, &mut blocks, &mut blocks_pending);

        while 0 < blocks_pending.len() {
            if (blocks_pending[0].0.id as usize) != blocks.len() {
                blocks_pending.remove(0);
                continue;
            }
            info!("Accepting block: {:#?}", blocks_pending[0].0);
            blocks.push(blocks_pending[0].0.clone());
            blocks_pending.remove(0);

            match tx_main_mint.send(Msg {
                command: Comm::EndMining,
                data: Vec::new(),
            }) {
                Ok(_) => {
                    (tx_main_mint, rx_main_mint) = unbounded::<Msg>();
                    debug!("Sending stop message to miner thread.");
                }
                Err(_) => {
                    warn!("Couldn't send stop message to miner thread.");
                }
            }
        }
    }
}
