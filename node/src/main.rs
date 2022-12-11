use chrono::Local;
use crossbeam_channel::unbounded;
use env_logger::Builder;
use gethostname::gethostname;
use lib::datatypes::{Block, Comm, Msg};
use lib::mint_block;
use lib::{handle_msg, networking::broadcast_chain, networking::listen};
use log::{debug, LevelFilter};
use std::io::Write;
use std::sync::mpsc;
use std::thread::sleep;
use std::thread::{self, JoinHandle};
use std::time::Duration;

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

    let node_name = match gethostname().into_string(){
        Ok(s) => {s}
        Err(_) => {panic!("Couldn't get hostname");}
    };

    let mut blocks: Vec<Block> = Vec::new();

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
                        (tx_main_mint, rx_main_mint) = unbounded::<Msg>();
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
                            debug!("Error during minting: {e}");
                        }
                    }
                }));
                is_miner_running = true;
                continue;
            }
            _ => {}
        }
        handle_msg(msg, &mut blocks, &tx_main_mint, &tx_listener);
    }
}
