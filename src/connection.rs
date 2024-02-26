use anyhow::Result;
use std::net::SocketAddrV4;
use tokio::net::TcpListener;

pub struct Connection {
  pub listener: TcpListener,
}

impl Connection {
  pub async fn new(ip: SocketAddrV4) -> Result<Self> {
    let listener = TcpListener::bind(ip).await?;
    Ok(Self { listener })
  }
}
