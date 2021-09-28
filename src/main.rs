use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use tokio::{net::UdpSocket, sync::mpsc};

#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    JoinRequest,
}

#[tokio::main]
pub async fn main() {
    let mut server = Server::new().await;

    server.update().await;
}

fn parse_request(bytes: &[u8]) -> Message {
    deserialize(bytes).unwrap()
}

fn serialize_request(msg: Message) -> Vec<u8> {
    serialize(&msg).unwrap()
}

struct Server {
    socket: Arc<UdpSocket>,
    clients: Vec<SocketAddr>,
}

impl Server {
    async fn new() -> Self {
        Self {
            socket: Arc::new(UdpSocket::bind("127.0.0.1:7878").await.unwrap()),
            clients: Vec::default(),
        }
    }

    async fn update(&mut self) {
        let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);

        let s = self.socket.clone();
        tokio::spawn(async move {
            while let Some((bytes, addr)) = rx.recv().await {
                Server::handle_response(s.clone(), addr, bytes).await;
            }
        });

        let mut buf = [0; 1024];
        loop {
            let (len, addr) = self.socket.recv_from(&mut buf).await.unwrap();
            println!("{:?} bytes received from {:?}", len, addr);
            tx.send((buf[..len].to_vec(), addr)).await.unwrap();
        }
    }

    async fn handle_response(socket: Arc<UdpSocket>, addr: SocketAddr, bytes: Vec<u8>) {
        println!("value: {}", String::from_utf8_lossy(&bytes));
        match parse_request(&bytes) {
            Message::JoinRequest => {}
            _ => {}
        }
    }
}
