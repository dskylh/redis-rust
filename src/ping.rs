use anyhow::Result;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn ping(stream: &mut TcpStream) -> Result<()> {
    let mut buf = [0; 128];
    loop {
        let size = stream.read(&mut buf)?;
        if size == 0 {
            return Ok(());
        }
        println!("Received: {}", String::from_utf8_lossy(&buf[..size]));
        stream.write_all(b"+PONG\r\n")?;
    }
}
