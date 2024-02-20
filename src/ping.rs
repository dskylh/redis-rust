use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

pub async fn ping(stream: &mut TcpStream) -> Result<()> {
    let mut buf = [0; 128];
    loop {
        let size = stream.read(&mut buf).await?;
        if size == 0 {
            return Ok(());
        }
        println!("Received: {}", String::from_utf8_lossy(&buf[..size]));
        stream.write_all(b"+PONG\r\n").await?;
    }
}
