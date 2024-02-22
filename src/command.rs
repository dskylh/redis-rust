use bytes::{Bytes, BytesMut};

use crate::parser::{RedisEncoder, RedisValueRef};

#[derive(Debug, PartialEq)]
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
                "echo" => RespCommand::Echo(parts.next().unwrap().into()),
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
        BytesMut::from("+PONG\r\n").into()
    }

    fn echo(&self, msg: &Bytes) -> Bytes {
        let mut encoder = RedisEncoder::default();
        let value = RedisValueRef::String(msg.clone());
        let mut buf = BytesMut::new();
        encoder.encode(value, &mut buf);
        buf.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_command() {
        let command = RespCommand::parse_command("ping");
        assert_eq!(command, RespCommand::Ping);

        let command = RespCommand::parse_command("echo hello");
        assert_eq!(command, RespCommand::Echo(Bytes::from("hello")));
    }

    #[test]
    fn test_execute() {
        let command = RespCommand::Ping;
        let response = command.execute();
        assert_eq!(response, Bytes::from("+PONG\r\n"));

        let command = RespCommand::Echo(Bytes::from("hello"));
        let response = command.execute();
        assert_eq!(response, Bytes::from("$5\r\nhello\r\n"));
    }
}
