use anyhow::anyhow;
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
    let listener = Connection::new(socket).await?.listener;
    loop {
        let (mut socket, _) = listener.accept().await?;

        // Spawn a new task for each connection to handle it concurrently
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                match socket.read(&mut buf).await {
                    // Return or break depending on your application logic
                    Ok(0) => return, // Connection closed
                    Ok(n) => {
                        // Process the received data
                        println!("Received: {}", String::from_utf8_lossy(&buf[..n]));
                    }
                    Err(e) => {
                        println!("Failed to read from socket; error = {:?}", e);
                        return;
                    }
                }
            }
        });
    }
    // loop {
    //     let listener = Connection::new(socket).await.unwrap().listener;
    //     let (mut connection, _) = listener.accept().await?;
    //     tokio::spawn(async move {
    //         let mut buf = BytesMut::new();
    //         loop {
    //             match connection.read(&mut buf).await {
    //                 Ok(0) => {
    //                     print!("000000000000000");
    //                     return Ok(());
    //                 }
    //                 Ok(n) => {
    //                     let request = String::from_utf8_lossy(&buf[..n]);
    //                     println!("Got data: {}", request);
    //                     let command = RespCommand::parse_command(&request);
    //                     let response = command.execute();
    //                     let bytes_written = connection.write_all(&response).await;
    //                     if bytes_written.is_err() {
    //                         return Err(anyhow!("Failed to write to socket"));
    //                     }
    //                     println!("Sent data: {}", String::from_utf8_lossy(&response));
    //                 }
    //                 Err(e) => {
    //                     return Err(anyhow!("Failed to read from socket; error = {:?}", e));
    //                 }
    //             }
    //         }
    //     });
    // }
}
