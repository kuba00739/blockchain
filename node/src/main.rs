use lib::listen;
use lib::Block;
use lib::Comm;
use lib::Msg;
use lib::Vin;
//use lib::send_message;
use bincode::{deserialize, serialize};
use lib::Car;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::env;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use String;

const HASH_LEN: usize = 32;

fn calculate_block(new_block: &mut Block, list_of_blocks: &Vec<Block>, nodes: &str) -> u8 {
    match list_of_blocks.last() {
        Some(last_block) => new_block.prev_hash = last_block.hash,
        None => new_block.prev_hash = [0; HASH_LEN],
    };
    new_block.id = list_of_blocks.len() as u32;
    drop(list_of_blocks);
    let calculated = mine_block(new_block);
    new_block.nonce = calculated.0;
    new_block.hash = calculated.1;
    return publish_block(new_block, nodes);
}

fn mine_block(new_block: &mut Block) -> (u32, [u8; HASH_LEN]) {
    let mut bytes: Vec<u8> = Vec::new();

    bytes.extend(&new_block.id.to_be_bytes());
    bytes.extend(&new_block.prev_hash);
    bytes.extend(&serialize(&new_block.registered_car).unwrap());

    let mut nonce: u32 = 0;

    while 1 == 1 {
        let mut sha2_hash = Sha256::new();
        sha2_hash.update(&bytes);
        sha2_hash.update(nonce.to_be_bytes());
        let sum = sha2_hash.finalize();
        if (sum[0] == 0) && (sum[1] == 0) {
            let result = match sum.try_into() {
                Err(cause) => panic!("Can't convert a result hash to a slice: {cause}"),
                Ok(result) => result,
            };
            return (nonce, result);
        };
        nonce += 1;
    }
    (0, [0; HASH_LEN])
}

fn publish_block(block: &Block, nodes: &str) -> u8 {
    let mut node_count: u8 = 0;
    for node in nodes.split(",") {
        let msg = Msg {
            command: Comm::NewBlock,
            data: serialize(block).unwrap(),
        };

        let stream: TcpStream;

        match TcpStream::connect(node) {
            Ok(s) => {
                stream = s;
            }
            Err(e) => {
                eprintln!("Error connecting to node {node}, {e}");
                continue;
            }
        };

        let buf = match lib::send_message(stream, msg) {
            Ok(s) => s,
            Err(_) => {
                continue;
            }
        };

        match deserialize::<Msg>(&buf) {
            Ok(s) => {
                println!("{:?}", s);
                match s.command {
                    Comm::Accepted => {
                        node_count += 1;
                    }
                    _ => {}
                }
            }
            Err(e) => {
                eprintln!("Error while reading response: {e}");
            }
        }
    }
    return node_count;
}

fn main() {
    let nodes = env::var("NODES").expect("Couldn't access NODES env variable.");
    let nodes_vec: Vec<&str> = nodes.split(",").collect();

    let blocks: Arc<Mutex<Vec<Block>>> = Arc::new(Mutex::new(Vec::new()));

    let listener_thread = thread::spawn({
        let blocks_clone = blocks.clone();
        move || {
            listen(blocks_clone);
        }
    });

    let new_car = Car::new(
        Some(String::from("Jakub")),
        Some(String::from("Niezabitowski")),
        Some(10000),
        Some(Vin::new(None, None, None)),
    );

    let one_more_car = Car::new(
        Some(String::from("Jakub")),
        Some(String::from("Niezabitowski")),
        Some(130000),
        Some(Vin::new(
            Some("2HG".to_string()),
            Some("C482G3".to_string()),
            Some("3A114352".to_string()),
        )),
    );

    let last_car = Car::new(
        Some(String::from("Third")),
        Some(String::from("Car")),
        Some(130000),
        Some(Vin::new(
            Some("2HG".to_string()),
            Some("C482G3".to_string()),
            Some("3A114352".to_string()),
        )),
    );

    let mut block = Block {
        hash: [0; HASH_LEN],
        id: 0,
        prev_hash: [0; HASH_LEN],
        nonce: 0,
        registered_car: new_car,
    };

    let mut block2 = Block {
        hash: [0; HASH_LEN],
        id: 0,
        prev_hash: [0; HASH_LEN],
        nonce: 0,
        registered_car: one_more_car,
    };

    let mut block3 = Block {
        hash: [0; HASH_LEN],
        id: 0,
        prev_hash: [0; HASH_LEN],
        nonce: 0,
        registered_car: last_car,
    };

    let mut rng = rand::thread_rng();
    thread::sleep(Duration::new(rng.gen_range(2..10), 0));

    if (calculate_block(
        &mut block,
        &blocks.lock().expect("Coulnd't lock block"),
        &nodes,
    ) as f64)
        / (nodes_vec.len() as f64)
        >= 0.5
    {
        blocks.lock().expect("Couldn't block").push(block);
    }

    thread::sleep(Duration::new(rng.gen_range(0..10), 0));
    if (calculate_block(&mut block2, &blocks.lock().expect("Couln't block"), &nodes) as f64)
        / (nodes_vec.len() as f64)
        >= 0.5
    {
        blocks.lock().expect("msLOOOOLg").push(block2);
    }

    thread::sleep(Duration::new(rng.gen_range(0..10), 0));
    if (calculate_block(&mut block3, &blocks.lock().expect("Couln't block"), &nodes) as f64)
        / (nodes_vec.len() as f64)
        >= 0.5
    {
        blocks.lock().expect("msLOOOOLg").push(block3);
    }

    //thread::sleep(Duration::new(rng.gen_range(0..10), 0));
    //println!("Blocks {:?}", &blocks.lock().expect("Couldn't lock file"));
    //drop(blocks);

    println!(
        "Final len: {}",
        blocks.lock().expect("Failed to lock").len()
    );

    match listener_thread.join() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error while joining listener thread: {:?}", e);
        }
    }
}
