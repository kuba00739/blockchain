use bincode::serialize;
use lib::send_all;
use lib::Car;
use lib::Comm;
use lib::Msg;
use std::env;

fn main() {
    let nodes = env::var("NODES").expect("Couldn't access NODES env variable.");
    let nodes_vec: Vec<&str> = nodes.split(",").collect();

    let new_car = Car::new(
        Some("Jakub".to_string()),
        Some("Niezabitowski".to_string()),
        Some(10000),
        None,
    );

    let argv: Vec<String> = env::args().collect();

    if &argv[1] == "DUMP" {
        send_all(
            Msg {
                command: Comm::PrintChain,
                data: Vec::new(),
            },
            &nodes_vec,
        )
    } else {
        send_all(
            Msg {
                command: Comm::DataToBlock,
                data: serialize(&new_car).unwrap(),
            },
            &nodes_vec,
        );
    }
}
