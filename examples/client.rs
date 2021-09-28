use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use tokio::{net::UdpSocket, sync::mpsc};

pub enum Message {}

#[tokio::main]
pub async fn main() {
    let socket = UdpSocket::bind("127.0.0.1:8008").await.unwrap();
    // let r = Arc::new(socket);
    // let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);

    socket
        .send_to("hello world".as_bytes(), "127.0.0.1:7878")
        .await
        .unwrap();
}
