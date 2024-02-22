use anyhow::Result;
use std::net::SocketAddrV4;
use tokio::net::{TcpListener, TcpStream};

pub struct Connection {
    pub stream: TcpStream,
}

impl Connection {
    pub async fn new(ip: SocketAddrV4) -> Result<Self> {
        let listener = TcpListener::bind(ip).await?;
        let (stream, _) = listener.accept().await?;
        Ok(Self { stream })
    }

    // pub async fn handle_client(&mut self) -> Result<()> {
    // let mut buf = BytesMut::new();
    // let mut parser = RespParser::default();
    // loop {
    //     match self.stream.read(&mut buf).await {
    //         // Ok(n) if n == 0 => {
    //         //     return Err(anyhow!("connection closed by client"));
    //         Ok(..) => {
    //             println!("{}", from_utf8(&buf)?);
    //             let command_str = parser.decode(&mut buf);
    //             match command_str {
    //                 Ok(Some(RedisValueRef::Array(arr))) => {
    //                     if let Some(RedisValueRef::String(str)) = arr.get(0) {
    //                         let response = from_utf8(str)?;
    //                         let command = RespCommand::parse_command(response);
    //                         let response = command.execute();
    //                         let _ = self.stream.write_all(&response);
    //                     }
    //                 }
    //                 Ok(None) => {
    //                     continue;
    //                 }
    //                 Err(..) => {
    //                     return Err(anyhow!("error parsing command"));
    //                 }
    //                 _ => unimplemented!(),
    //             }
    //         }
    //         Err(e) => {
    //             println!("Something happened {e}")
    //         }
    //     }
    // }
    // }
    // }
}
