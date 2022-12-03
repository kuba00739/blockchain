use lib::broadcast_chain;
use lib::handle_msg;
use lib::listen;
use lib::Block;
use lib::Car;
use lib::Comm;
use lib::Msg;
use std::env;
use std::sync::mpsc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

const HASH_LEN: usize = 32;

fn main() {
    let (tx_listener, rx_main) = mpsc::channel::<Msg>();

    let nodes = env::var("NODES").expect("Couldn't access NODES env variable.");
    let node_name = env::var("NAME").expect("Couldn't access NODES env variable.");

    let nodes_vec: Vec<&str> = nodes.split(",").collect();

    let mut blocks: Vec<Block> = Vec::new();
    let mut block_pending: (Block, u8) = (
        Block {
            hash: [0; HASH_LEN],
            id: 0,
            prev_hash: [0; HASH_LEN],
            nonce: 0,
            registered_car: Car::new(None, None, None, None),
            mined_by: "".to_string(),
        },
        0,
    );

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
        println!("Received msg: {:?}", msg);
        match msg.command {
            Comm::Broadcast => {
                broadcast_chain(&blocks, &nodes_vec);
            }
            _ => {}
        }
        handle_msg(msg, &mut blocks, &nodes_vec, &mut block_pending, &node_name);
        if (block_pending.1 as f64) / (nodes_vec.len() as f64) >= 0.5 {
            println!("Accepting block: {:?}", block_pending.0);
            blocks.push(block_pending.0.clone());
            block_pending.1 = 0;
        }
    }
}
