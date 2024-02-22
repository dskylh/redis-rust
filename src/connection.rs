use std::{net::SocketAddrV4, str::from_utf8};

use crate::{
    command::RespCommand,
    parser::{RedisValueRef, RespParser},
};
use anyhow::{anyhow, Result};
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    pub async fn new(ip: SocketAddrV4) -> Result<Self> {
        let listener = TcpListener::bind(ip).await?;
        let (stream, _) = listener.accept().await?;
        Ok(Self { stream })
    }

    pub async fn handle_client(&mut self) -> Result<()> {
        let mut buf = BytesMut::new();
        let mut parser = RespParser::default();
        loop {
            match self.stream.read(&mut buf).await {
                Ok(n) if n == 0 => {
                    return Err(anyhow!("connection closed by client"));
                }
                Ok(..) => {
                    let command_str = parser.decode(&mut buf);
                    match command_str {
                        Ok(Some(RedisValueRef::Array(arr))) => {
                            if let Some(RedisValueRef::String(str)) = arr.get(0) {
                                let response = from_utf8(str)?;
                                let command = RespCommand::parse_command(response);
                                let response = command.execute();
                                let _ = self.stream.write_all(&response);
                            }
                        }
                        Ok(None) => {
                            continue;
                        }
                        Err(..) => {
                            return Err(anyhow!("error parsing command"));
                        }
                        _ => unimplemented!(),
                    }
                }
                Err(..) => {}
            }
        }
    }

    pub async fn read_command(&mut self) -> Result<RespCommand> {
        let mut buf = BytesMut::new();
        loop {
            let size = self.stream.read(&mut buf).await?;
            if size < 0 {
                return Err(anyhow!("stream no good"));
            }
            let mut parser = RespParser::default();
            let response = parser.decode(&mut buf).unwrap().unwrap();
            if let RedisValueRef::Array(arr) = response {
                if let Some(RedisValueRef::String(str)) = arr.get(0) {
                    let response = from_utf8(str)?;
                    return Ok(RespCommand::parse_command(response));
                }
            }
        }
    }
}
