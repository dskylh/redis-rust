use anyhow::anyhow;
use std::{
  env,
  net::{Ipv4Addr, SocketAddrV4},
  sync::{Arc, Mutex},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use bytes::BytesMut;
use redis_starter_rust::{
  command::{RespCommand, Store},
  connection::Connection,
  parser::{RedisValueRef, RespParser},
};

// handle arguments for selecting the port
// the port argument will be given as such --port <port_number>
fn handle_args(args: Vec<String>) -> anyhow::Result<SocketAddrV4> {
  let mut port = 6379;
  for (i, arg) in args.iter().enumerate() {
    if arg == "--port" {
      if i + 1 < args.len() {
        port = args[i + 1].parse().unwrap();
      } else {
        return Err(anyhow!("no port given"));
      }
    }
  }
  Ok(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // You can use print statements as follows for debugging, they'll be visible when running tests.
  println!("Logs from your program will appear here!");

  let args: Vec<String> = env::args().collect();

  let socket = handle_args(args)?;
  let listener = Connection::new(socket).await?.listener;
  let mut parser = RespParser::default();

  let store: Arc<Mutex<Store>> = Arc::new(Mutex::new(Store::new()));
  loop {
    let (mut socket, _) = listener.accept().await?;
    let store = Arc::clone(&store);

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
            match parser.decode(&mut buf).unwrap() {
              Some(value) => {
                if let RedisValueRef::Array(arr) = value {
                  let response = RespCommand::parse_command_arr(arr);
                  let response = response.execute(store.lock().unwrap().clone());

                  let bytes_written = socket.write_all(&response).await;
                  if bytes_written.is_err() {
                    return Err(anyhow!("error happened while writing to socket"));
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
