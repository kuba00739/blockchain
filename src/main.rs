use bincode::{serialize, deserialize};
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest,};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
//use tracing_mutex::stdsync::{TracingMutex};
use std::sync::{Arc, Mutex};
use std::thread;
use std::env;
use std::time::Duration;


use String;

const HASH_LEN: usize = 32;
const NODES: [&str; 2] = ["127.0.0.1:9091", "127.0.0.1:9093"];
const NODE_AMOUNT: u8 = 3;


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

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
enum Comm {
    NewBlock,
    Accepted,
    Rejected
}


#[derive(Serialize, Deserialize)]
#[derive(Debug)]
struct Msg {
    command: Comm,
    data: Vec<u8>
}

fn calculate_block(new_block: &mut Block, list_of_blocks: &Vec<Block>) -> u8{
    match list_of_blocks.last() {
        Some(last_block) => new_block.prev_hash = last_block.hash,
        None => new_block.prev_hash = [0;HASH_LEN],
    };
    new_block.id = list_of_blocks.len() as u32;
    drop(list_of_blocks);
    let calculated = mine_block(new_block);
    new_block.nonce = calculated.0;
    new_block.hash = calculated.1;
    return publish_block(new_block);
}

fn mine_block(new_block: &mut Block) -> (u32, [u8;HASH_LEN]){
    let mut bytes: Vec<u8> = Vec::new();

    bytes.extend(&new_block.id.to_be_bytes());
    bytes.extend(&new_block.prev_hash);
    bytes.extend(&serialize(&new_block.registered_car).unwrap());

    let mut nonce: u32 = 0;

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


fn verify_block(data: &Vec<u8>, mut stream: TcpStream, blockchain: Arc<Mutex<Vec<Block>>>) {
    let block = deserialize::<Block>(data).expect("Error while reading block from message");

    println!("Verifying block: {:?}", block);

    let mut bytes: Vec<u8> = Vec::new();

    bytes.extend(&block.id.to_be_bytes());

    let control_prev_hash = match blockchain.try_lock().expect("Couldn't lock blockchain").last() {
        Some(last_block) => last_block.hash,
        None => [0;HASH_LEN],
    };

    if control_prev_hash != block.prev_hash{
        let msg = Msg{
            command: Comm::Rejected,
            data: Vec::new()
        };
        match stream.write(&serialize(&msg).unwrap()){
            Ok(_) => {}
            Err(e) => {eprintln!("Failed to send a message: {e}");}
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

    if (sum[0]==0) && (sum[1]==0){
        blockchain.lock().expect("Block accepted but lock failed").push(block);
        msg = Msg{
            command: Comm::Accepted,
            data: Vec::new()
        };
    } else{
        msg = Msg{
            command: Comm::Rejected,
            data: Vec::new()
        };
    }
    match stream.write(&serialize(&msg).unwrap()){
        Ok(_) => {println!("Sent {:?} successfuly.", msg.command);}
        Err(e) => {eprintln!("Error while writing response: {e}");}
    }
}

fn handle_incoming(mut stream: TcpStream, blockchain: Arc<Mutex<Vec<Block>>>){
    let mut buff = [0;1280];
    match stream.read(&mut buff) {
        Ok(_d) => {}
        Err(e) => {eprintln!("Error while handling stream: {e}");}
    }

    match deserialize::<Msg>(&buff){
        Ok(s) => {
            println!("Received message: {:?}", s);
            match s.command{
                Comm::NewBlock => {verify_block(&s.data, stream, blockchain);},
                Comm::Rejected => {println!("Woow, rejected 2");}
                _ => {println!("Lmao");}
            };
        
        },
        Err(e) => {eprintln!("Error while deserializing message: {e}");}
    };
}


fn listen(addr: &String, blockchain: Arc<Mutex<Vec<Block>>>){
    let listener = TcpListener::bind(addr).unwrap();

    for stream in listener.incoming(){
        let stream = stream.unwrap();
        let peer_addr = stream.peer_addr().unwrap();
        println!("Remote connection from {:#?}", peer_addr);

        let thr = thread::spawn({
            let blockchain_clone = blockchain.clone();
            move || {
                handle_incoming(stream, blockchain_clone);
            }
        });
        match thr.join(){
            Ok(_s) => {println!("Remote connection with {:#?} closed", peer_addr);}
            Err(e) => {eprintln!("Error while joining thread: {:#?}", e);}

        };
        println!("Remote connection with {:#?} closed", peer_addr);
    }

}

fn send_message(mut stream: TcpStream, msg: Msg) -> Result<[u8; 1280],&'static str>{
    let mut buf = [0; 1280]; 

    stream.set_write_timeout(Some(Duration::new(5, 0))).expect("Couldn't set timeout");
    match stream.write(&serialize(&msg).unwrap()){
        Ok(_s) => {}
        Err(e) => {
            eprintln!("Error while writing to stream: {e}");
            return Err("Error connecting to node")
        }
    }
    
    //stream.set_read_timeout(Some(Duration::new(5, 0))).unwrap();
    match stream.read(&mut buf) {
        Ok(_s) => {Ok(buf)}
        Err(e) => {eprintln!("Error while reading data from stream: {e}"); Err("Error reading data")}
    }
}

fn publish_block(block: &Block) -> u8{
    let mut node_count: u8 = 0;
    for node in NODES{
        let msg = Msg{
            command: Comm::NewBlock,
            data: serialize(block).unwrap()
        };

        let stream :TcpStream;

        match TcpStream::connect(node){
            Ok(s) => {stream = s;}
            Err(e) => {
                eprintln!("Error connecting to node {node}, {e}");
                continue;
            }
        };

        let buf = send_message(stream, msg).expect("Couldn't publish block to one of the nodes");

        match deserialize::<Msg>(&buf){
            Ok(s) => {
                println!("{:?}", s); 
                match s.command{
                    Comm::Accepted => {node_count += 1;}
                    _ => {}
                }
        }
            Err(e) => {eprintln!("Error while reading response: {e}");}
        }
    }
    return node_count;
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let blocks: Arc<Mutex<Vec<Block>>>  = Arc::new(Mutex::new(Vec::new()));

    let listener_thread = thread::spawn(
        {
            let blocks_clone = blocks.clone();
            move || {
                listen(&args[1], blocks_clone);
            }
        }
    );


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

    if (calculate_block(&mut block, &blocks.try_lock().expect("Coulnd't lock block")) as f64)/(NODE_AMOUNT as f64) >= 0.5 {
        blocks.try_lock().expect("Couldn't block").push(block);
    }
    if (calculate_block(&mut block2, &blocks.try_lock().expect("Couln't block")) as f64)/(NODE_AMOUNT as f64) >= 0.5 {
        blocks.try_lock().expect("msLOOOOLg").push(block2);
    }


    //drop(blocks);

    match listener_thread.join(){
        Ok(_) => {}
        Err(e) => {eprintln!("Error while joining listener thread: {:?}", e);}
    }


    println!("{:?}", blocks.lock().expect("Failed to lock"));
    

}
