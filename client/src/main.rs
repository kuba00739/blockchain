use bincode::serialize;
use lib::Car;
use lib::Comm;
use lib::Msg;
use rand::Rng;
use std::env;
use std::net::UdpSocket;

static NAMES: [&str; 10] = [
    "James", "Oliver", "Max", "Muller", "Bravo", "Fox", "Jimmy", "Jakub", "Willy", "Billy",
];

fn main() {
    let mut rng = rand::thread_rng();

    let new_car = Car::new(
        Some(NAMES[rng.gen_range(0..9)].to_string()),
        Some(NAMES[rng.gen_range(0..9)].to_string()),
        Some(rng.gen_range(0..1000000)),
        None,
    );

    let argv: Vec<String> = env::args().collect();
    let socket: UdpSocket = UdpSocket::bind("192.168.128.253:8000").expect("Error while binding");

    if &argv[1] == "DUMP" {
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
    } else {
        println!(
            "Broadcasted {} bytes",
            socket
                .send_to(
                    &serialize(&Msg {
                        command: Comm::DataToBlock,
                        data: serialize(&new_car).unwrap(),
                    })
                    .expect("Error serializing"),
                    "239.0.0.1:9000"
                )
                .expect("Error sending message")
        );
    }
}
