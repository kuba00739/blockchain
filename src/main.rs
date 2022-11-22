use bincode::serialize;
use serde::{Serialize};

use String;

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
#[derive(Serialize)]
struct Block {
    hash: u32,
    id: u32,
    prev_hash: u32,
    nonce: u32,
    registered_car: Car,
}

fn calculate_block(new_block: &mut Block, list_of_blocks: &Vec<Block>) {
    let prev_hash;
    match list_of_blocks.last() {
        Some(block) => prev_hash = block.hash,
        None => prev_hash = 0,
    };
    new_block.prev_hash = prev_hash;
    new_block.id = list_of_blocks.len() as u32;
    let calculated = mine_block(new_block);
    new_block.nonce = calculated.0;
    new_block.hash = calculated.1;
}

fn mine_block(new_block: &mut Block) -> (u32, u32){
    let mut bytes: Vec<u8> = Vec::new();
    bytes.extend(&new_block.id.to_be_bytes());
    bytes.extend(&new_block.prev_hash.to_be_bytes());
    bytes.extend(&new_block.nonce.to_be_bytes());
    let test_bytes = serialize(&new_block.registered_car);
    bytes.extend(&test_bytes.unwrap());

    let mut sum: u32 = 1;
    let mut nonce: u32 = 0;
    while (sum % 1000) != 0 {
        sum = 0;
        for byte in &bytes{
            sum += *byte as u32;
        }
        sum += nonce;
        nonce += 1; 
    }
    nonce -= 1;
    (nonce, sum)
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
    println!("New Car: {:?}", new_car);
    let mut block= Block{
        hash: 0,
        id: 0,
        prev_hash: 0,
        nonce: 0,
        registered_car: new_car
    };

    calculate_block(&mut block, &blocks);
    blocks.push(block);

    println!("{:?}", blocks);
    //println!("Block: {:?}", &block);

}
