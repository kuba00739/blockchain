use crate::{Block, Comm, Msg};
use bincode::serialize;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use crate::verify_block;
use bincode::deserialize;

use std::sync::{Arc, Mutex};

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

pub fn listen(addr: &String, blockchain: Arc<Mutex<Vec<Block>>>) {
    let listener = TcpListener::bind(addr).unwrap();

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
