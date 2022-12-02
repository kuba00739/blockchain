use bincode::deserialize;
use bincode::serialize;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use std::sync::{Arc, Mutex};

const HASH_LEN: usize = 32;

#[derive(Debug, Serialize, Deserialize)]
pub struct Vin {
    wmi: String,
    vds: String,
    vis: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Car {
    owner_name: String,
    owner_surname: String,
    distance_traveled: u32,
    vin_number: Vin,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub hash: [u8; HASH_LEN],
    pub id: u32,
    pub prev_hash: [u8; HASH_LEN],
    pub nonce: u32,
    pub registered_car: Car,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Comm {
    NewBlock,
    Accepted,
    Rejected,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct Msg {
    pub command: Comm,
    pub data: Vec<u8>,
}

impl Car {
    pub fn new(
        owner_name: Option<String>,
        owner_surname: Option<String>,
        distance_traveled: Option<u32>,
        vin_number: Option<Vin>,
    ) -> Car {
        Car {
            owner_name: owner_name.unwrap_or("".to_string()),
            owner_surname: owner_surname.unwrap_or("".to_string()),
            distance_traveled: distance_traveled.unwrap_or(0),
            vin_number: vin_number.unwrap_or(Vin::new(None, None, None)),
        }
    }
}

impl Vin {
    pub fn new(wmi: Option<String>, vds: Option<String>, vis: Option<String>) -> Vin {
        Vin {
            wmi: wmi.unwrap_or_else(|| "".to_string()),
            vds: vds.unwrap_or_else(|| "".to_string()),
            vis: vis.unwrap_or_else(|| "".to_string()),
        }
    }
}

fn verify_block(data: &Vec<u8>, mut stream: TcpStream, blockchain: Arc<Mutex<Vec<Block>>>) {
    let block = deserialize::<Block>(data).expect("Error while reading block from message");

    println!("Verifying block: {:?}", block);

    let mut bytes: Vec<u8> = Vec::new();

    bytes.extend(&block.id.to_be_bytes());

    let control_prev_hash = match blockchain.lock().expect("Couldn't lock blockchain").last() {
        Some(last_block) => last_block.hash,
        None => [0; HASH_LEN],
    };

    if control_prev_hash != block.prev_hash {
        let msg = Msg {
            command: Comm::Rejected,
            data: Vec::new(),
        };
        match stream.write(&serialize(&msg).unwrap()) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to send a message: {e}");
            }
        }
        return;
    }

    drop(control_prev_hash);

    bytes.extend(&block.prev_hash);
    bytes.extend(&serialize(&block.registered_car).unwrap());

    let mut sha2_hash = Sha256::new();
    sha2_hash.update(&bytes);
    sha2_hash.update(block.nonce.to_be_bytes());
    let sum = sha2_hash.finalize();

    let msg: Msg;

    if (sum[0] == 0) && (sum[1] == 0) {
        blockchain
            .lock()
            .expect("Block accepted but lock failed")
            .push(block);
        msg = Msg {
            command: Comm::Accepted,
            data: Vec::new(),
        };
    } else {
        msg = Msg {
            command: Comm::Rejected,
            data: Vec::new(),
        };
    }
    match stream.set_write_timeout(Some(Duration::new(5, 0))) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error while sending response {e}");
        }
    }
    match stream.write(&serialize(&msg).unwrap()) {
        Ok(_) => {
            println!("Sent {:?} successfuly.", msg.command);
        }
        Err(e) => {
            eprintln!("Error while writing response: {e}");
        }
    }
}

pub fn send_message(mut stream: TcpStream, msg: Msg) -> Result<[u8; 1280], &'static str> {
    let mut buf = [0; 1280];

    stream
        .set_write_timeout(Some(Duration::new(5, 0)))
        .expect("Couldn't set timeout");
    match stream.write(&serialize(&msg).unwrap()) {
        Ok(_s) => {}
        Err(e) => {
            eprintln!("Error while writing to stream: {e}");
            return Err("Error connecting to node");
        }
    }

    stream.set_read_timeout(Some(Duration::new(5, 0))).unwrap();
    match stream.read(&mut buf) {
        Ok(_s) => Ok(buf),
        Err(e) => {
            eprintln!("Error while reading data from stream: {e}");
            Err("Error reading data")
        }
    }
}

pub fn listen(blockchain: Arc<Mutex<Vec<Block>>>) {
    let listener = TcpListener::bind("0.0.0.0:9000").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let peer_addr = stream.peer_addr().unwrap();
        println!("Remote connection from {:#?}", peer_addr);

        let thr = thread::spawn({
            let blockchain_clone = blockchain.clone();
            move || {
                handle_incoming(stream, blockchain_clone);
            }
        });
        match thr.join() {
            Ok(_s) => {
                println!("Remote connection with {:#?} closed", peer_addr);
            }
            Err(e) => {
                eprintln!("Error while joining thread: {:#?}", e);
            }
        };
        println!("Remote connection with {:#?} closed", peer_addr);
    }
}

fn handle_incoming(mut stream: TcpStream, blockchain: Arc<Mutex<Vec<Block>>>) {
    let mut buff = [0; 1280];
    match stream.read(&mut buff) {
        Ok(_d) => {}
        Err(e) => {
            eprintln!("Error while handling stream: {e}");
        }
    }

    match deserialize::<Msg>(&buff) {
        Ok(s) => {
            println!("Received message: {:?}", s);
            match s.command {
                Comm::NewBlock => {
                    verify_block(&s.data, stream, blockchain);
                }
                Comm::Rejected => {
                    println!("Woow, rejected 2");
                }
                _ => {
                    println!("Lmao");
                }
            };
        }
        Err(e) => {
            eprintln!("Error while deserializing message: {e}");
        }
    };
}
