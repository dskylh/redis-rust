use std::net::{Ipv4Addr, SocketAddrV4};

// Uncomment this block to pass the first stage
use redis_starter_rust::connection::Connection;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let socket = SocketAddrV4::new(ip, 6379);
    let mut connection = Connection::new(socket).await?;
}
