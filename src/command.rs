use crate::parser::{RedisEncoder, RedisValueRef};
use anyhow::anyhow;
use bytes::{Bytes, BytesMut};
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

#[derive(Debug, Clone)]
pub struct Store {
  data: Arc<Mutex<HashMap<Bytes, Bytes>>>,
}

impl Store {
  pub fn new() -> Self {
    let data: Arc<Mutex<HashMap<Bytes, Bytes>>> = Arc::new(Mutex::new(HashMap::new()));

    Self { data }
  }

  pub fn set(&self, key: Bytes, value: Bytes) -> anyhow::Result<()> {
    if let Ok(mut data) = self.data.lock() {
      data.insert(key, value);
      Ok(())
    } else {
      Err(anyhow!("couldn't set the data"))
    }
  }

  pub fn get(&self, key: &Bytes) -> Option<Bytes> {
    let data = self.data.lock().unwrap();
    data.get(key).cloned()
  }
}

#[derive(Debug)]
pub enum Response {
  Bytes(Bytes),
  Db(Store),
}

#[derive(Debug)]
pub enum RespCommand {
  Ping,
  Echo(Bytes),
  Set((Bytes, Bytes)),
  Get(Bytes),
}

impl RespCommand {
  pub fn parse_command(command: &Bytes, argument: Option<Bytes>) -> RespCommand {
    match String::from_utf8_lossy(&command).to_lowercase().as_ref() {
      "ping" => RespCommand::Ping,
      "echo" => RespCommand::Echo(argument.unwrap()),
      _ => panic!("unknown command"),
    }
  }

  pub fn parse_command_arr(args: Vec<RedisValueRef>) -> RespCommand {
    let mut command: Bytes = Bytes::new();
    let mut arguments: Vec<Bytes> = Vec::new();
    for (i, value) in args.iter().enumerate() {
      if i == 0 {
        if let RedisValueRef::String(c) = value {
          command = c.clone();
        }
      } else {
        if let RedisValueRef::String(a) = value {
          arguments.push(a.clone());
        }
      }
    }
    match String::from_utf8_lossy(&command).to_lowercase().as_ref() {
      "ping" => RespCommand::Ping,
      "echo" => RespCommand::Echo(arguments[0].clone()),
      "set" => RespCommand::Set((arguments[0].clone(), arguments[1].clone())),
      "get" => RespCommand::Get(arguments[0].clone()),
      _ => panic!("unknown command"),
    }
  }

  pub fn execute(&self, store: Store) -> Bytes {
    match self {
      RespCommand::Ping => self.ping(),
      RespCommand::Echo(msg) => self.echo(&msg),
      RespCommand::Set((key, value)) => self.set(key, value, store),
      RespCommand::Get(key) => self.get(key, store),
    }
  }

  fn set(&self, key: &Bytes, value: &Bytes, store: Store) -> Bytes {
    match store.set(key.clone(), value.clone()) {
      Ok(_) => Bytes::from("+OK\r\n"),
      Err(_) => Bytes::from("-ERR\r\n"),
    }
  }

  fn get(&self, key: &Bytes, store: Store) -> Bytes {
    let mut encoder = RedisEncoder::default();
    match store.get(key) {
      Some(value) => {
        let value = RedisValueRef::String(value);
        let mut buf = BytesMut::new();
        encoder.encode(value, &mut buf);
        buf.into()
      }
      None => Bytes::from("$-1\r\n"),
    }
  }

  fn ping(&self) -> Bytes {
    Bytes::from("+PONG\r\n")
  }

  fn echo(&self, msg: &Bytes) -> Bytes {
    let mut encoder = RedisEncoder::default();
    let value = RedisValueRef::String(msg.clone());
    let mut buf = BytesMut::new();
    encoder.encode(value, &mut buf);
    buf.into()
  }
}

// #[cfg(test)]
// mod tests {
//   use super::*;
//
//   #[test]
//   fn test_parse_command() {
//     // let command = RespCommand::parse_command(b"ping", None);
//     // assert_eq!(command, RespCommand::Ping);
//     //
//     // let command = RespCommand::parse_command("echo hello");
//     // assert_eq!(command, RespCommand::Echo(Bytes::from("hello")));
//   }
//
//   #[test]
//   fn test_execute() {
//     let command = RespCommand::Ping;
//     let response = command.execute();
//     assert_eq!(response, Bytes::from("+PONG\r\n"));
//
//     let command = RespCommand::Echo(Bytes::from("hello"));
//     let response = command.execute();
//     assert_eq!(response, Bytes::from("$5\r\nhello\r\n"));
//   }
// }
