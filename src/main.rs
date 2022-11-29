use bincode::serialize;
use serde::{Serialize};
use sha2::{Sha256, Digest,};

use String;

const HASH_LEN: usize = 32;

#[derive(Debug)]
#[derive(Serialize)]
struct Vin {
    wmi: String,
    vds: String,
    vis: String,
}
#[derive(Debug)]
#[derive(Serialize)]
struct Car {
    owner_name: String,
    owner_surname: String,
    distance_traveled: u32,
    vin_number: Vin,
}
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

fn main() {
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

    //println!("{:?}", block);

    calculate_block(&mut block, &blocks);
    blocks.push(block);
    calculate_block(&mut block2, &blocks);
    blocks.push(block2);

    println!("{:?}", blocks);
    //println!("Block: {:?}", &block);

}
