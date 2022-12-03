use lib::handle_msg;
use lib::listen;
use lib::Block;
use lib::Car;
use lib::Msg;
use std::env;
use std::sync::mpsc;
use std::thread;

const HASH_LEN: usize = 32;

fn main() {
    let (tx_listener, rx_main) = mpsc::channel::<Msg>();

    let nodes = env::var("NODES").expect("Couldn't access NODES env variable.");
    let nodes_vec: Vec<&str> = nodes.split(",").collect();

    let mut blocks: Vec<Block> = Vec::new();
    let mut block_pending: (Block, u8) = (
        Block {
            hash: [0; HASH_LEN],
            id: 0,
            prev_hash: [0; HASH_LEN],
            nonce: 0,
            registered_car: Car::new(None, None, None, None),
        },
        0,
    );

    let tx1 = tx_listener.clone();

    thread::spawn({
        move || {
            listen(tx1);
        }
    });

    for msg in rx_main {
        println!("Received msg: {:?}", msg);
        handle_msg(msg, &blocks, &nodes_vec, &mut block_pending);
        if (block_pending.1 as f64) / (nodes_vec.len() as f64) > 0.5 {
            blocks.push(block_pending.0.clone());
            block_pending = (
                Block {
                    hash: [0; HASH_LEN],
                    id: 0,
                    prev_hash: [0; HASH_LEN],
                    nonce: 0,
                    registered_car: Car::new(None, None, None, None),
                },
                0,
            );
        }
    }
}
