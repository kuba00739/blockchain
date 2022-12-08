use crate::{Block, Comm, Msg};
use bincode::{deserialize, serialize};
use log::{debug, warn};
use std::net::{Ipv4Addr, UdpSocket};
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;

pub fn listen(tx: Sender<Msg>) {
    let listener = UdpSocket::bind("0.0.0.0:9000").unwrap();
    listener
        .join_multicast_v4(
            &Ipv4Addr::from_str("239.0.0.1").unwrap(),
            &Ipv4Addr::UNSPECIFIED,
        )
        .expect("Error while joining multicast v4");
    let mut threads: Vec<JoinHandle<()>> = Vec::new();
    loop {
        let mut bytes: Vec<u8> = vec![0; 4096];
        let (len, addr) = match listener.recv_from(&mut bytes) {
            Ok(s) => s,
            Err(e) => {
                warn!("Error in recv_from: {e}");
                return;
            }
        };
        debug!("Remote connection from {:#?}, {} bytes read.", addr, len);

        threads.push(thread::spawn({
            let tx1 = tx.clone();
            move || match handle_incoming(bytes, tx1) {
                Ok(_) => {}
                Err(e) => {
                    warn!("Error while handling incoming message: {e}")
                }
            }
        }));
    }
}

fn handle_incoming(bytes: Vec<u8>, tx: Sender<Msg>) -> Result<(), Box<dyn std::error::Error>> {
    let msg = deserialize::<Msg>(&bytes)?;
    debug!("Received message: {:#?}", msg);
    tx.send(msg)?;
    Ok(())
}

pub fn send_all(msg: Msg) -> Result<(), Box<dyn std::error::Error>> {
    let socket: UdpSocket = UdpSocket::bind("0.0.0.0:8000")?;

    let bytes = socket.send_to(&serialize(&msg)?, "239.0.0.1:9000")?;
    debug!("Broadcasted {} bytes", bytes);

    Ok(())
}

pub fn broadcast_chain(blockchain: &Vec<Block>) {
    match send_all(Msg {
        command: Comm::Blockchain,
        data: serialize(blockchain).unwrap(),
    }) {
        Ok(_) => {}
        Err(e) => {
            warn!("Error during broadcasting chain: {e}");
        }
    }
}
