use bincode::{serialize, deserialize};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest,};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::env;

use String;

const HASH_LEN: usize = 32;
const NODES: [&str; 3] = ["127.0.0.1:9091", "127.0.0.1:9092", "127.0.0.1:9093"];

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct Vin {
    wmi: String,
    vds: String,
    vis: String,
}
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
struct Car {
    owner_name: String,
    owner_surname: String,
    distance_traveled: u32,
    vin_number: Vin,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct Block {
    hash: [u8; HASH_LEN],
    id: u32,
    prev_hash: [u8; HASH_LEN], 
    nonce: u32,
    registered_car: Car,
}

fn calculate_block(new_block: &mut Block, list_of_blocks: &Vec<Block>) {
    match list_of_blocks.last() {
        Some(last_block) => new_block.prev_hash = last_block.hash,
        None => new_block.prev_hash = [0;HASH_LEN],
    };
    new_block.id = list_of_blocks.len() as u32;
    let calculated = mine_block(new_block);
    new_block.nonce = calculated.0;
    new_block.hash = calculated.1;
    publish_block(new_block);
}

fn mine_block(new_block: &mut Block) -> (u32, [u8;HASH_LEN]){
    let mut bytes: Vec<u8> = Vec::new();

    bytes.extend(&new_block.id.to_be_bytes());
    bytes.extend(&new_block.prev_hash);
    bytes.extend(&new_block.nonce.to_be_bytes());
    bytes.extend(&serialize(&new_block.registered_car).unwrap());

    let mut nonce: u32 = 0;
    //let mut finished = false;

    while 1==1 {
        let mut sha2_hash = Sha256::new();
        sha2_hash.update(&bytes);
        sha2_hash.update(nonce.to_be_bytes());
        let sum = sha2_hash.finalize();
        if (sum[0]==0) && (sum[1]==0) {
            let result = match sum.try_into(){
                Err(cause) => panic!("Can't convert a result hash to a slice: {cause}"),
                Ok(result) => result,
            };
            return (nonce, result);
        };
        nonce += 1; 
    };
    (0, [0;HASH_LEN])
}


fn handle_incoming(mut stream: TcpStream){
    let mut buff = [0;1280];
    match stream.read(&mut buff) {
        Ok(_d) => {}
        Err(e) => {eprintln!("Error while handling stream: {e}");}
    }
    println!("{:?}", deserialize::<Block>(&buff).unwrap());
}


fn send(ip: &str, data: &[u8]){
    let mut stream :TcpStream;
    match TcpStream::connect(ip){
        Ok(s) => {stream = s;}
        Err(e) => {
            eprintln!("Error connecting to node {ip}, {e}");
            return;
        }
    };
    match stream.write(data){
        Ok(_s) => {return;}
        Err(e) => {
            eprintln!("Error while writing to stream: {e}");
            return;
        }
    }
}


fn listen(addr: &String){
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming(){
        let stream = stream.unwrap();
        let peer_addr = stream.peer_addr().unwrap();
        println!("Remote connection from {:#?}", peer_addr);

        let thr = thread::spawn(|| {
            handle_incoming(stream);
        });
        match thr.join(){
            Ok(_s) => {println!("Remote connection with {:#?} closed", peer_addr);}
            Err(e) => {eprintln!("Error while joining thread: {:#?}", e);}

        };
        println!("Remote connection with {:#?} closed", peer_addr);
    }

}

fn publish_block(block: &Block){
    for node in NODES{
        send(node, &serialize(block).unwrap());
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut blocks: Vec<Block> = Vec::new();
    let new_car = Car{
        owner_name: String::from("Jakub"),
        owner_surname: String::from("Niezabitowski"),
        distance_traveled: 10000,
        vin_number: Vin{
            wmi: "1HG".to_string(),
            vds: "CM8263".to_string(),
            vis: "3A004352".to_string(),
        }
    };

    let one_more_car = Car{
        owner_name: String::from("Jakub"),
        owner_surname: String::from("Niezabitowski"),
        distance_traveled: 130000,
        vin_number: Vin{
            wmi: "2HG".to_string(),
            vds: "C482G3".to_string(),
            vis: "3A114352".to_string(),
        }
    };

    println!("New Car: {:?}", new_car);
    let mut block= Block{
        hash: [0;HASH_LEN],
        id: 0,
        prev_hash: [0;HASH_LEN],
        nonce: 0,
        registered_car: new_car
    };

    let mut block2= Block{
        hash: [0; HASH_LEN],
        id: 0,
        prev_hash: [0;HASH_LEN],
        nonce: 0,
        registered_car: one_more_car
    };

    calculate_block(&mut block, &blocks);
    blocks.push(block);
    calculate_block(&mut block2, &blocks);
    blocks.push(block2);

    println!("{:?}", blocks);
    let listener_thread = thread::spawn(move || {
        listen(&args[1]);
    });

    for node in NODES{
        send(node, &serialize(&blocks[0]).unwrap());
    }
    listener_thread.join().expect("Error while joining listener thread.");


}
