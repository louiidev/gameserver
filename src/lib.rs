use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr, sync::{Arc, Mutex}};
use tokio::{net::UdpSocket, sync::mpsc};

pub fn parse_request(bytes: &[u8]) -> Message {
    deserialize(bytes).unwrap()
}

pub fn serialize_request(msg: Message) -> Vec<u8> {
    serialize(&msg).unwrap()
}


#[derive(Debug, Deserialize, Serialize)]
pub enum Message {
    JoinRequest,
}

pub struct Server {
    socket: Arc<UdpSocket>,
    clients: Arc<Mutex<Vec<SocketAddr>>>,
}

impl Server {
    pub async fn new() -> Self {
        Self {
            socket: Arc::new(UdpSocket::bind("127.0.0.1:7878").await.unwrap()),
            clients: Arc::new(Mutex::new(Vec::default())),
        }
    }

    pub async fn update(&mut self) {
        let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);

        let s = self.socket.clone();
        let c = self.clients.clone();
        tokio::spawn(async move {
            while let Some((bytes, addr)) = rx.recv().await {
                Server::handle_response(s.clone(), c.clone(), addr, bytes).await;
            }
        });

        let mut buf = [0; 1024];
        loop {
            let (len, addr) = self.socket.recv_from(&mut buf).await.unwrap();
            println!("{:?} bytes received from {:?}", len, addr);
            println!("clients: {}", self.clients.lock().unwrap().len());
            tx.send((buf[..len].to_vec(), addr)).await.unwrap();
        }
    }

    async fn handle_response(socket: Arc<UdpSocket>, clients: Arc<Mutex<Vec<SocketAddr>>>, addr: SocketAddr, bytes: Vec<u8>) {
        match parse_request(&bytes) {
            Message::JoinRequest => {
                clients.lock().unwrap().push(addr);
            },
            _ => {}
        }
    }
}