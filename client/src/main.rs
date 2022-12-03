use bincode::serialize;
use lib::send_all;
use lib::Car;
use lib::Comm;
use lib::Msg;
use rand::Rng;
use std::env;

static NAMES: [&str; 10] = [
    "James", "Oliver", "Max", "Muller", "Bravo", "Fox", "Jimmy", "Jakub", "Willy", "Billy",
];

fn main() {
    let nodes = match env::var("NODES") {
        Ok(s) => s,
        Err(_) => {
            panic!("MISSING NODES ENVVAR");
        }
    };

    let mut rng = rand::thread_rng();
    let nodes_vec: Vec<&str> = nodes.split(",").collect();

    let new_car = Car::new(
        Some(NAMES[rng.gen_range(0..9)].to_string()),
        Some(NAMES[rng.gen_range(0..9)].to_string()),
        Some(rng.gen_range(0..1000000)),
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
