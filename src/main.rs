use gameserver::Server;

#[tokio::main]
pub async fn main() {
    let mut server = Server::new().await;

    server.update().await;
}
