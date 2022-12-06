use chrono::Local;
use env_logger::Builder;
use lib::{handle_msg, networking::broadcast_chain, networking::listen};
use lib::{Block, Comm, Msg};
use log::LevelFilter;
use log::{debug, info};
use std::env;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use std::thread::sleep;
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

    for msg in rx_main {
        debug!("Received msg: {:#?}", msg);
        match msg.command {
            Comm::Broadcast => {
                broadcast_chain(&blocks);
            }
            _ => {}
        }
        handle_msg(msg, &mut blocks, &mut blocks_pending, &node_name);

        let mut index: usize = 0;

        while index < blocks_pending.len() {
            if (blocks_pending[index].0.id as usize) != blocks.len() {
                blocks_pending.remove(index);
                continue;
            }
            if blocks_pending[index].1 >= 2 {
                info!("Accepting block: {:#?}", blocks_pending[index].0);
                blocks.push(blocks_pending[index].0.clone());
                blocks_pending.remove(index);
                continue;
            }

            index += 1;
        }
    }
}
