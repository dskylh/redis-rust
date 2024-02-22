use bytes::{Bytes, BytesMut};
use core::{fmt, str};
use memchr::memchr;
use std::fmt::Display;

pub type Value = Bytes;
pub type Key = Bytes;

pub const NULL_ARRAY: &str = "*-1\r\n";
pub const NULL_BULK_STRING: &str = "*-1\r\n";
#[derive(PartialEq, Clone, Debug)]
pub enum RedisValueRef {
    String(Bytes),
    Error(Bytes),
    Int(i64),
    Array(Vec<RedisValueRef>),
    NullArray,
    NullBulkString,
}

impl Display for RedisValueRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RedisValueRef::String(s) => write!(f, "String({})", str::from_utf8(s).unwrap()),
            RedisValueRef::Error(e) => write!(f, "Error({})", str::from_utf8(e).unwrap()),
            RedisValueRef::Int(i) => write!(f, "Int({})", i),
            RedisValueRef::Array(a) => write!(f, "Array({:?})", a),
            RedisValueRef::NullArray => write!(f, "NullArray"),
            RedisValueRef::NullBulkString => write!(f, "NullBulkString"),
        }
    }
}

#[derive(Debug)]
struct BufSplit(usize, usize);

impl<'a> BufSplit {
    fn as_slice(&'a self, buf: &'a BytesMut) -> &[u8] {
        &buf[self.0..self.1]
    }
    fn as_bytes(&'a self, buf: &Bytes) -> Bytes {
        buf.slice(self.0..self.1)
    }
}

/// BufSplit based equivalent to our output type RedisValueRef
#[derive(Debug)]
enum RedisBufSplit {
    String(BufSplit),
    Error(BufSplit),
    Int(i64),
    Array(Vec<RedisBufSplit>),
    NullArray,
    NullBulkString,
}

impl RedisBufSplit {
    fn redis_value(self, buf: &Bytes) -> RedisValueRef {
        match self {
            // bfs is BufSplit(start, end), which has the as_bytes method defined above
            RedisBufSplit::String(bfs) => RedisValueRef::String(bfs.as_bytes(buf)),
            RedisBufSplit::Error(bfs) => RedisValueRef::Error(bfs.as_bytes(buf)),
            RedisBufSplit::Array(arr) => {
                RedisValueRef::Array(arr.into_iter().map(|bfs| bfs.redis_value(buf)).collect())
            }
            RedisBufSplit::NullArray => RedisValueRef::NullArray,
            RedisBufSplit::NullBulkString => RedisValueRef::NullBulkString,
            RedisBufSplit::Int(i) => RedisValueRef::Int(i),
        }
    }
}

#[derive(Debug)]
pub enum RESPError {
    UnexpectedEnd,
    UnknownStartingByte,
    IOError(std::io::Error),
    IntParseFailure,
    BadBulkStringSize(i64),
    BadArraySize(i64),
}

impl fmt::Display for RESPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            RESPError::UnexpectedEnd => write!(f, "Unexpected end of input"),
            RESPError::UnknownStartingByte => write!(f, "Unknown starting byte"),
            RESPError::IOError(ref e) => write!(f, "IOError: {}", e),
            RESPError::IntParseFailure => write!(f, "Failed to parse integer"),
            RESPError::BadBulkStringSize(size) => write!(f, "Bad bulk string size: {}", size),
            RESPError::BadArraySize(size) => write!(f, "Bad array size: {}", size),
        }
    }
}

type RedisResult = Result<Option<(usize, RedisBufSplit)>, RESPError>;

fn word(buf: &BytesMut, pos: usize) -> Option<(usize, BufSplit)> {
    // We're at the edge of `buf`, so we can't find a word.
    if buf.len() <= pos {
        return None;
    }
    // Find the position of the b'\r'
    memchr(b'\r', &buf[pos..]).and_then(|end| {
        if end + 1 < buf.len() {
            // pos + end == first index of b'\r' after `pos`
            // pos + end + 2 == ..word\r\n<HERE> -- skip to after CLRF
            Some((pos + end + 2, BufSplit(pos, pos + end)))
        } else {
            // Edge case: We received just enough bytes from the client
            // to get the \r but not the \n
            None
        }
    })
}

fn simple_string(buf: &BytesMut, pos: usize) -> RedisResult {
    if let Some((pos, word)) = word(buf, pos) {
        Ok(Some((pos, RedisBufSplit::String(word))))
    } else {
        Ok(None)
    }
}

fn simple_error(buf: &BytesMut, pos: usize) -> RedisResult {
    if let Some((pos, word)) = word(buf, pos) {
        Ok(Some((pos, RedisBufSplit::Error(word))))
    } else {
        Ok(None)
    }
}

fn int(buf: &BytesMut, pos: usize) -> Result<Option<(usize, i64)>, RESPError> {
    if let Some((pos, word)) = word(buf, pos) {
        // word.as_slice(buf) is the method call BufSplit::as_slice(&self, &BytesMut) to access the byte slice.
        let s = str::from_utf8(word.as_slice(buf)).map_err(|_| RESPError::IntParseFailure)?;
        // Convert the string to an i64. Note the `?` for early returns.
        let i = s.parse().map_err(|_| RESPError::IntParseFailure)?;
        Ok(Some((pos, i)))
    } else {
        Ok(None)
    }
}

fn resp_int(buf: &BytesMut, pos: usize) -> RedisResult {
    Ok(int(buf, pos)?.map(|(pos, int)| (pos, RedisBufSplit::Int(int))))
}

fn bulk_string(buf: &BytesMut, pos: usize) -> RedisResult {
    match int(buf, pos)? {
        Some((pos, -1)) => Ok(Some((pos, RedisBufSplit::NullBulkString))),
        // size is more than 0
        Some((pos, size)) if size >= 0 => {
            let total_size = pos + size as usize;
            // if buffer is smaller than the size of the bulk string +2 for crlf
            if buf.len() < total_size + 2 {
                Ok(None)
            } else {
                let bb = RedisBufSplit::String(BufSplit(pos, total_size));
                Ok(Some((total_size + 2, bb)))
            }
        }
        Some((_pos, bad_size)) => Err(RESPError::BadBulkStringSize(bad_size)),
        None => Ok(None),
    }
}

fn array(buf: &BytesMut, pos: usize) -> RedisResult {
    match int(buf, pos)? {
        None => Ok(None),
        Some((pos, -1)) => Ok(Some((pos, RedisBufSplit::NullArray))),
        Some((pos, size)) if size >= 0 => {
            let mut elements = Vec::with_capacity(size as usize);

            let mut curr = pos;

            for _ in 0..size {
                if let Some((new_pos, value)) = parse(buf, curr)? {
                    curr = new_pos;
                    elements.push(value);
                } else {
                    return Ok(None);
                }
            }
            Ok(Some((curr, RedisBufSplit::Array(elements))))
        }
        Some((_pos, bad_num_elements)) => Err(RESPError::BadArraySize(bad_num_elements)),
    }
}

fn parse(buf: &BytesMut, pos: usize) -> RedisResult {
    if buf.is_empty() {
        return Ok(None);
    }
    match buf[pos] {
        b'+' => simple_string(buf, pos + 1),
        b'-' => simple_error(buf, pos + 1),
        b':' => resp_int(buf, pos + 1),
        b'$' => bulk_string(buf, pos + 1),
        b'*' => array(buf, pos + 1),
        _ => Err(RESPError::UnknownStartingByte),
    }
}

#[derive(Default)]
pub struct RespParser;

impl RespParser {
    pub fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<RedisValueRef>, RESPError> {
        if buf.is_empty() {
            return Ok(None);
        }

        match parse(buf, 0)? {
            Some((pos, value)) => {
                // We parsed a value! Shave off the bytes so tokio can continue filling the buffer.
                let our_data = buf.split_to(pos);
                // Use `redis_value` defined above to get the correct type
                Ok(Some(value.redis_value(&our_data.freeze())))
            }
            None => Ok(None),
        }
    }
}

#[derive(Default)]
pub struct RedisEncoder;

impl RedisEncoder {
    pub fn encode(&mut self, item: RedisValueRef, dst: &mut BytesMut) {
        write_redis_value(item, dst);
    }
}

fn write_redis_value(item: RedisValueRef, dst: &mut BytesMut) {
    match item {
        RedisValueRef::Error(e) => {
            dst.extend_from_slice(b"-");
            dst.extend_from_slice(&e);
            dst.extend_from_slice(b"\r\n");
        }
        RedisValueRef::Error(e) => {
            dst.extend_from_slice(b"-");
            dst.extend_from_slice(&e);
            dst.extend_from_slice(b"\r\n");
        }
        RedisValueRef::String(s) => {
            dst.extend_from_slice(b"$");
            dst.extend_from_slice(s.len().to_string().as_bytes());
            dst.extend_from_slice(b"\r\n");
            dst.extend_from_slice(&s);
            dst.extend_from_slice(b"\r\n");
        }
        RedisValueRef::Array(array) => {
            dst.extend_from_slice(b"*");
            dst.extend_from_slice(array.len().to_string().as_bytes());
            dst.extend_from_slice(b"\r\n");
            for redis_value in array {
                write_redis_value(redis_value, dst);
            }
        }
        RedisValueRef::Int(i) => {
            dst.extend_from_slice(b":");
            dst.extend_from_slice(i.to_string().as_bytes());
            dst.extend_from_slice(b"\r\n");
        }
        RedisValueRef::NullArray => dst.extend_from_slice(NULL_ARRAY.as_bytes()),
        RedisValueRef::NullBulkString => dst.extend_from_slice(NULL_BULK_STRING.as_bytes()),
    }
}

#[cfg(test)]
mod resp_decode_test {
    use super::{RedisValueRef, RespParser};
    use bytes::{Bytes, BytesMut};
    // input: &str, expected_output: RedisValueRef
    fn test_decoder(input: &str, expected_output: RedisValueRef) {
        let mut decoder = RespParser::default();
        let mut buf = BytesMut::from(input);

        let output = decoder.decode(&mut buf).unwrap();

        if let Some(o) = output {
            assert_eq!(o, expected_output)
        }
    }

    #[test]
    fn test_simple_string() {
        test_decoder(
            "+hello world\r\n",
            RedisValueRef::String(Bytes::from("hello world")),
        );
    }

    #[test]
    fn test_simple_error() {
        test_decoder(
            "-ERR this is an error\r\n",
            RedisValueRef::Error(Bytes::from("ERR this is an error")),
        );
    }

    #[test]
    fn test_integer() {
        test_decoder(":1000\r\n", RedisValueRef::Int(1000));
    }

    #[test]
    fn test_null_bulk_string() {
        test_decoder("$-1\r\n", RedisValueRef::NullBulkString);
    }

    #[test]
    fn test_bulk_string() {
        test_decoder(
            "$11\r\nhello world\r\n",
            RedisValueRef::String(Bytes::from("hello world")),
        );
    }

    #[test]
    fn test_null_array() {
        test_decoder("*-1\r\n", RedisValueRef::NullArray);
    }

    #[test]
    fn test_array() {
        test_decoder(
            "*2\r\n+hello\r\n:1000\r\n",
            RedisValueRef::Array(vec![
                RedisValueRef::String(Bytes::from("hello")),
                RedisValueRef::Int(1000),
            ]),
        );
    }

    #[test]
    fn test_nested_array() {
        test_decoder(
            "*2\r\n*2\r\n+hello\r\n:1000\r\n*2\r\n+world\r\n:2000\r\n",
            RedisValueRef::Array(vec![
                RedisValueRef::Array(vec![
                    RedisValueRef::String(Bytes::from("hello")),
                    RedisValueRef::Int(1000),
                ]),
                RedisValueRef::Array(vec![
                    RedisValueRef::String(Bytes::from("world")),
                    RedisValueRef::Int(2000),
                ]),
            ]),
        );
    }

    #[test]
    fn test_empty_array() {
        test_decoder("*0\r\n", RedisValueRef::Array(vec![]));
    }
}
