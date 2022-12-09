use bincode::serialize;
use lib::BlockData;
use lib::Car;
use lib::Comm;
use lib::Msg;
use lib::RevPolish;
use rand::Rng;
use std::env;
use std::net::UdpSocket;

static NAMES: [&str; 10] = [
    "James", "Oliver", "Max", "Muller", "Bravo", "Fox", "Jimmy", "Jakub", "Willy", "Billy",
];

fn main() {
    let mut rng = rand::thread_rng();

    let data = BlockData::Car(Car::new(
        Some(NAMES[rng.gen_range(0..9)].to_string()),
        Some(NAMES[rng.gen_range(0..9)].to_string()),
        Some(rng.gen_range(0..1000000)),
        None,
    ));

    let argv: Vec<String> = env::args().collect();
    let socket: UdpSocket = UdpSocket::bind("192.168.128.253:8000").expect("Error while binding");

    if argv.len() < 2 {
        println!("Please provide at least one argument:\nDUMP\nCAR\nCONT");
        return;
    }

    match argv[1].to_uppercase().as_str() {
        "DUMP" => {
            println!(
                "Broadcasted {} bytes",
                socket
                    .send_to(
                        &serialize(&Msg {
                            command: Comm::PrintChain,
                            data: Vec::new(),
                        })
                        .expect("Error serializing"),
                        "239.0.0.1:9000"
                    )
                    .expect("Error sending message")
            );
            return;
        }
        "CAR" => {
            println!(
                "Broadcasted {} bytes",
                socket
                    .send_to(
                        &serialize(&Msg {
                            command: Comm::DataToBlock,
                            data: serialize(&data).unwrap(),
                        })
                        .expect("Error serializing"),
                        "239.0.0.1:9000"
                    )
                    .expect("Error sending message")
            );
        }
        "CONT" => {
            let mut contract: Vec<RevPolish> = Vec::new();
            for i in &argv[2..] {
                if i.as_str() == "+" {
                    contract.push(RevPolish::Operation('+'));
                } else if i.as_str() == "-" {
                    contract.push(RevPolish::Operation('-'));
                } else if i.as_str() == "*" {
                    contract.push(RevPolish::Operation('*'));
                } else if i.as_str() == "a" {
                    contract.push(RevPolish::Arg);
                } else {
                    contract.push(RevPolish::Number(
                        i.as_str().parse::<i32>().expect("Unexpected string!"),
                    ));
                }
            }

            let data2 = BlockData::Contract(contract);

            println!(
                "Broadcasted {} bytes",
                socket
                    .send_to(
                        &serialize(&Msg {
                            command: Comm::DataToBlock,
                            data: serialize(&data2).unwrap(),
                        })
                        .expect("Error serializing"),
                        "239.0.0.1:9000"
                    )
                    .expect("Error sending message")
            );
        }
        _ => {
            println!("Invalid argument.");
            return;
        }
    }
}
