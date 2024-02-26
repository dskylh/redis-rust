use anyhow::anyhow;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use bytes::BytesMut;
// Uncomment this block to pass the first stage
use redis_starter_rust::{
  command::RespCommand,
  connection::Connection,
  parser::{RedisEncoder, RedisValueRef, RespParser},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // You can use print statements as follows for debugging, they'll be visible when running tests.
  println!("Logs from your program will appear here!");

  let ip = Ipv4Addr::new(127, 0, 0, 1);
  let socket = SocketAddrV4::new(ip, 6379);
  let listener = Connection::new(socket).await?.listener;
  let mut parser = RespParser::default();
  let _encoder = RedisEncoder::default();
  loop {
    let (mut socket, _) = listener.accept().await?;

    // Spawn a new task for each connection to handle it concurrently
    tokio::spawn(async move {
      let mut buf = BytesMut::new();
      buf.resize(1024, 0);
      loop {
        match socket.read(&mut buf).await {
          // Return or break depending on your application logic
          Ok(0) => return Ok(()), // Connection closed
          Ok(_n) => {
            // Process the received data
            println!("Received: {}", String::from_utf8_lossy(&buf.clone()));
            match parser.decode(&mut buf).unwrap() {
              Some(value) => {
                if let RedisValueRef::Array(arr) = value {
                  if arr.len() == 1 {
                    if let RedisValueRef::String(c) = &arr[0] {
                      let command = RespCommand::parse_command(&c, None);
                      let response = command.execute();
                      println!("{}", String::from_utf8_lossy(&response));
                      let bytes_written = socket.write_all(&response).await;
                      if bytes_written.is_err() {
                        return Err(anyhow!("error happened while writing to socket"));
                      }
                    };
                  } else if arr.len() == 2 {
                    if let RedisValueRef::String(c) = &arr[0] {
                      if let RedisValueRef::String(arg) = &arr[1] {
                        let command = RespCommand::parse_command(&c, Some(arg.clone()));
                        let response = command.execute();
                        println!("{}", String::from_utf8_lossy(&response));
                        let bytes_written = socket.write_all(&response).await;
                        if bytes_written.is_err() {
                          return Err(anyhow!("error happened while writing to socket"));
                        }
                      }
                    }
                  }
                }
              }
              None => {
                continue;
              }
            }
          }
          Err(e) => {
            println!("Failed to read from socket; error = {:?}", e);
            return Err(anyhow!("error happened: {}", e));
          }
        }
      }
    });
  }
}
