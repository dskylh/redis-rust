use bytes::{Bytes, BytesMut};

use crate::parser::{RedisEncoder, RedisValueRef};

pub enum RespCommand {
    Ping,
    Echo(Bytes),
}

impl RespCommand {
    pub fn parse_command(command: &str) -> RespCommand {
        let parts = command.split_whitespace();
        let mut parts = parts.map(|s| s.to_owned().to_lowercase());
        match parts.next() {
            Some(cmd) => match cmd.as_str() {
                "ping" => RespCommand::Ping,
                "echo" => {
                    RespCommand::Echo(parts.collect::<Vec<String>>().join(" ").as_bytes().into())
                }
                _ => panic!("Unknown command"),
            },
            None => panic!("No command"),
        }
    }

    pub fn execute(&self) -> Bytes {
        match self {
            RespCommand::Ping => self.ping(),
            RespCommand::Echo(msg) => self.echo(&msg),
        }
    }

    fn ping(&self) -> Bytes {
        let mut encoder = RedisEncoder::default();
        let value = RedisValueRef::String("+PONG".as_bytes().into());
        let mut buf = BytesMut::new();
        encoder.encode(value, &mut buf);
        buf.into()
    }

    fn echo(&self, msg: &Bytes) -> Bytes {
        let mut encoder = RedisEncoder::default();
        let value = RedisValueRef::String(msg.clone());
        let mut buf = BytesMut::new();
        encoder.encode(value, &mut buf);
        buf.into()
    }
}
