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

fn send_data(socket: UdpSocket, msg: Msg) {
    println!(
        "Broadcasted {} bytes",
        socket
            .send_to(
                &serialize(&msg).expect("Error serializing"),
                "239.0.0.1:9000"
            )
            .expect("Error sending message")
    );
}

fn main() {
    let mut rng = rand::thread_rng();

    let argv: Vec<String> = env::args().collect();
    let socket: UdpSocket = UdpSocket::bind("192.168.128.253:8000").expect("Error while binding");

    if argv.len() < 2 {
        println!("Please provide at least one argument:\nDUMP\nCAR\nCONT");
        return;
    }

    match argv[1].to_uppercase().as_str() {
        "DUMP" => {
            send_data(
                socket,
                Msg {
                    command: Comm::PrintChain,
                    data: Vec::new(),
                },
            );
            return;
        }
        "CAR" => {
            let data = BlockData::Car(Car::new(
                Some(NAMES[rng.gen_range(0..9)].to_string()),
                Some(NAMES[rng.gen_range(0..9)].to_string()),
                Some(rng.gen_range(0..1000000)),
                None,
            ));
            send_data(
                socket,
                Msg {
                    command: Comm::DataToBlock,
                    data: serialize(&data).unwrap(),
                },
            );
        }
        "CONT" => {
            let mut contract: Vec<RevPolish> = Vec::new();
            let operations = ['+', '-', '*', '%', '/', 'p'];
            for i in &argv[2..] {
                if operations.contains(&i.as_str().chars().next().unwrap()) {
                    contract.push(RevPolish::Operation(i.as_str().chars().next().unwrap()));
                } else if i.as_str() == "a" {
                    contract.push(RevPolish::Arg);
                } else {
                    contract.push(RevPolish::Number(
                        i.as_str().parse::<f64>().expect("Unexpected string!"),
                    ));
                }
            }

            let data = BlockData::Contract(contract);

            send_data(
                socket,
                Msg {
                    command: Comm::DataToBlock,
                    data: serialize(&data).unwrap(),
                },
            );
        }
        "CALC" => {
            let mut args: Vec<f64> = Vec::new();
            for i in &argv[2..] {
                args.push(i.parse().expect("Unexpected string!"));
            }

            send_data(
                socket,
                Msg {
                    command: Comm::CalcContract,
                    data: serialize(&args).unwrap(),
                },
            );
        }
        _ => {
            println!("Invalid argument.");
            return;
        }
    }
}
