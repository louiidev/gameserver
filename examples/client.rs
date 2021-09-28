use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use gameserver::{Message, serialize_request};

use bincode::serialize;
use tokio::{net::UdpSocket, sync::mpsc};


#[tokio::main]
pub async fn main() {
    let socket = UdpSocket::bind("127.0.0.1:8008").await.unwrap();
    // let r = Arc::new(socket);
    // let (tx, mut rx) = mpsc::channel::<(Vec<u8>, SocketAddr)>(1_000);

    socket
        .send_to(&serialize_request(Message::JoinRequest), "127.0.0.1:7878")
        .await
        .unwrap();
}
