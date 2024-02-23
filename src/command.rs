use bytes::{Bytes, BytesMut};

use crate::parser::{RedisEncoder, RedisValueRef};

#[derive(Debug, PartialEq)]
pub enum RespCommand {
  Ping,
  Echo(Bytes),
}

impl RespCommand {
  pub fn parse_command(command: &Bytes, argument: Option<Bytes>) -> RespCommand {
    match String::from_utf8_lossy(&command).to_lowercase().as_ref() {
      "ping" => RespCommand::Ping,
      "echo" => RespCommand::Echo(argument.unwrap()),
      _ => panic!("unknown command"),
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
    // let command = RespCommand::parse_command(b"ping", None);
    // assert_eq!(command, RespCommand::Ping);
    //
    // let command = RespCommand::parse_command("echo hello");
    // assert_eq!(command, RespCommand::Echo(Bytes::from("hello")));
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
