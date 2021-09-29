use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};
use smol_rs::math::Vector2;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
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
    JoinAccepted,
    InputState(InputState),
    Snapshot(SnapshotState),
    Disconnect,
    Error(String),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SnapshotState {
    pub player_position: (f32, f32),
}

type Clients = Arc<Mutex<HashMap<SocketAddr, ClientState>>>;

pub struct Server {
    socket: Arc<UdpSocket>,
    clients: Clients,
}

#[derive(Default)]
pub struct ClientState {
    position: (f32, f32),
}

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct InputState {
    pub direction: (f32, f32),
    pub delta: f32,
}

impl Server {
    pub async fn new() -> Self {
        Self {
            socket: Arc::new(UdpSocket::bind("127.0.0.1:7878").await.unwrap()),
            clients: Arc::new(Mutex::new(HashMap::new())),
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
            println!("looping..");

            match self.socket.try_recv_from(&mut buf) {
                Ok((len, addr)) => {
                    println!("{:?} bytes received from {:?}", len, addr);
                    tx.send((buf[..len].to_vec(), addr)).await.unwrap();
                }
                _ => {}
            }

            for client in self.clients.lock().unwrap().iter() {
                let s = self.socket.clone();
                Server::send_message(
                    s,
                    Message::Snapshot(SnapshotState {
                        player_position: client.1.position,
                    }),
                    *client.0,
                )
                .await;
            }
            // tokio::time::sleep(Duration::from_millis(33)).await;
        }
    }

    async fn send_message(socket: Arc<UdpSocket>, msg: Message, addr: SocketAddr) {
        socket.send_to(&serialize_request(msg), addr).await.unwrap();
    }

    async fn handle_response(
        socket: Arc<UdpSocket>,
        clients: Clients,
        addr: SocketAddr,
        bytes: Vec<u8>,
    ) {
        match parse_request(&bytes) {
            Message::Disconnect => {
                let mut clients = clients.lock().unwrap();
                if clients.contains_key(&addr) {
                    clients.remove(&addr);
                }
            }
            Message::InputState(InputState { direction, delta }) => {
                let mut clients = clients.lock().unwrap();
                let client = clients.get_mut(&addr).unwrap();
                client.position = (
                    client.position.0 + direction.0 * delta * 200.,
                    client.position.1 + direction.1 * delta * 200.,
                );
            }
            Message::JoinRequest => {
                let contains = {
                    let clients = clients.lock().unwrap();
                    clients.contains_key(&addr)
                };
                if contains {
                    Server::send_message(
                        socket,
                        Message::Error("User tried to join twice".into()),
                        addr,
                    )
                    .await;
                    panic!("User tried to join twice");
                } else {
                    {
                        let mut clients = clients.lock().unwrap();
                        clients.insert(addr, ClientState::default());
                    }
                    Server::send_message(socket, Message::JoinAccepted, addr).await;
                }
            }
            _ => {}
        }
    }
}

pub struct Client {
    socket: std::net::UdpSocket,
}

impl Client {
    pub fn new() -> Self {
        use std::net::UdpSocket;
        let socket = UdpSocket::bind("127.0.0.1:8008").unwrap();
        socket
            .send_to(&serialize_request(Message::JoinRequest), "127.0.0.1:7878")
            .unwrap();

        let mut buf = [0; 1024];
        socket.recv(&mut buf).unwrap();

        println!("{:?}", parse_request(&buf));

        socket.set_nonblocking(true).unwrap();

        Self { socket }
    }

    pub fn poll(&self) -> Option<SnapshotState> {
        let mut buf = [0; 1024];
        match self.socket.recv(&mut buf) {
            Ok(i) => {
                println!("{}", i)
            }
            _ => {}
        }

        Client::parse_server_request(buf)
    }

    fn parse_server_request(buf: [u8; 1024]) -> Option<SnapshotState> {
        let request = parse_request(&buf);

        match request {
            Message::Snapshot(state) => Some(state),
            _ => None,
        }
    }

    pub fn send_input(&self, input_state: InputState) {
        self.socket
            .send_to(
                &serialize_request(Message::InputState(input_state)),
                "127.0.0.1:7878",
            )
            .unwrap();
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.socket
            .send_to(&serialize_request(Message::Disconnect), "127.0.0.1:7878")
            .unwrap();
    }
}
