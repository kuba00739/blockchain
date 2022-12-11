use chrono::Local;
use crossbeam_channel::unbounded;
use env_logger::Builder;
use gethostname::gethostname;
use lib::datatypes::{Block, Msg};
use lib::{handle_msg, networking::listen};
use log::{debug, LevelFilter};
use std::io::Write;
use std::sync::mpsc;
use std::thread::sleep;
use std::thread::{self, JoinHandle};
use std::time::Duration;

fn main() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();

    let (tx_mpsc, rx_mpsc) = mpsc::channel::<Msg>();
    let (mut tx_mpmc, mut rx_mpmc) = unbounded::<Msg>();

    let node_name = match gethostname().into_string() {
        Ok(s) => s,
        Err(_) => {
            panic!("Couldn't get hostname");
        }
    };

    let mut blocks: Vec<Block> = Vec::new();

    let tx_mpsc_1 = tx_mpsc.clone();

    thread::spawn({
        move || {
            listen(tx_mpsc_1);
        }
    });

    let tx_mpsc_2 = tx_mpsc.clone();

    thread::spawn({
        move || loop {
            sleep(Duration::new(60, 0));
            tx_mpsc_2
                .send(Msg {
                    command: lib::Comm::Broadcast,
                    data: Vec::new(),
                })
                .expect("Message to main thread couldn't be sent.");
        }
    });

    let mut miner_thread: Option<JoinHandle<()>> = None;
    let mut is_miner_running: bool = false;

    for msg in rx_mpsc {
        debug!("Received msg: {:#?}", msg);
        match msg.command {
            _ => {
                handle_msg(
                    msg,
                    &mut blocks,
                    &mut is_miner_running,
                    &mut miner_thread,
                    &node_name,
                    &tx_mpsc,
                    &mut tx_mpmc,
                    &mut rx_mpmc,
                );
            }
        }
    }
}
