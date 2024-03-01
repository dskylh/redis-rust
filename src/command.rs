use crate::parser::{RedisEncoder, RedisValueRef};
use anyhow::anyhow;
use bytes::{Bytes, BytesMut};
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
  time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub struct Store {
  data: Arc<Mutex<HashMap<Bytes, Bytes>>>,
  expiry_times: Arc<Mutex<HashMap<Bytes, Instant>>>,
}

impl Store {
  pub fn new() -> Self {
    let data: Arc<Mutex<HashMap<Bytes, Bytes>>> = Arc::new(Mutex::new(HashMap::new()));
    let expiry_times: Arc<Mutex<HashMap<Bytes, Instant>>> = Arc::new(Mutex::new(HashMap::new()));

    Store { data, expiry_times }
  }

  pub fn set(&self, key: Bytes, value: Bytes, expiry: Option<Duration>) -> anyhow::Result<()> {
    let mut data = self.data.lock().unwrap();
    let mut expiry_times = self.expiry_times.lock().unwrap();

    data.insert(key.clone(), value.clone());
    if let Some(expiry) = expiry {
      expiry_times.insert(key, Instant::now() + expiry);
    }

    Ok(())
  }

  pub fn get(&self, key: &Bytes) -> Option<Bytes> {
    // only get a key if it didn't expire
    let expired = self
      .expiry_times
      .lock()
      .unwrap()
      .get(key)
      .map(|expiry_time| expiry_time > &Instant::now());

    if expired == Some(true) {
      self.data.lock().unwrap().remove(key);
      self.expiry_times.lock().unwrap().remove(key);
      None
    } else {
      self.data.lock().unwrap().get(key).cloned()
    }
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
  Set((Bytes, Bytes, Option<Bytes>)),
  Get(Bytes),
}

impl RespCommand {
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
      "set" => RespCommand::Set((
        arguments[0].clone(),
        arguments[1].clone(),
        arguments.get(3).cloned(),
      )),
      "get" => RespCommand::Get(arguments[0].clone()),
      _ => panic!("unknown command"),
    }
  }

  pub fn execute(&self, store: Store) -> Bytes {
    match self {
      RespCommand::Ping => self.ping(),
      RespCommand::Echo(msg) => self.echo(&msg),
      RespCommand::Set((key, value, expiry)) => self.set(key, value, expiry.clone(), store),
      RespCommand::Get(key) => self.get(key, store),
    }
  }

  fn set(&self, key: &Bytes, value: &Bytes, expiry: Option<Bytes>, store: Store) -> Bytes {
    let expiry = match expiry {
      Some(expiry) => {
        let expiry = String::from_utf8_lossy(&expiry).parse().unwrap();
        println!("{:?}", expiry);
        Some(Duration::from_secs(expiry))
      }
      None => None,
    };

    match store.set(key.clone(), value.clone(), expiry) {
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
