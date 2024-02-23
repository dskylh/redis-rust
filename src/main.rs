use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

use bytes::BytesMut;
// Uncomment this block to pass the first stage
use redis_starter_rust::{command::RespCommand, connection::Connection};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let socket = SocketAddrV4::new(ip, 6379);
    loop {
        let listener = Connection::new(socket).await.unwrap().listener;
        let (mut connection, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = BytesMut::new();
            connection.read(&mut buf).await.unwrap();
            let request = String::from_utf8_lossy(&buf);
            println!("Got data: {}", request);
            let command = RespCommand::parse_command(&request);
            let response = command.execute();
            let bytes_written = connection.write_all(&response).await;
            if bytes_written.is_err() {
                return;
            }
            println!("Sent data: {}", String::from_utf8_lossy(&response));
        });
    }
}
