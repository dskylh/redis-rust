use anyhow::anyhow;
use bytes::Bytes;
// const CRLF: &[u8; 2] = b"\r\n";
const SIM_STRING: u8 = b'+';
// const SIM_ERRORS: u8 = b'-';
const INTEGER: u8 = b':';
const BULK_STRING: u8 = b'$';
const ARRAY: u8 = b'*';

#[derive(Debug, PartialEq)]
pub enum Resp {
    SimpleString(String),
    SimpleError(String),
    Integer(i64),
    BulkString(Bytes),
    Array(Vec<Resp>),
}

impl Resp {
    pub fn parse_simple_string(input: Bytes) -> anyhow::Result<Resp> {
        let mut buf = String::new();
        let consumed = 0;

        if let Some(first) = input.first() {
            if first != &SIM_STRING {
                return Err(anyhow!("Not a correct string byte"));
            }
        }

        let mut iter = input.iter().skip(1).peekable();

        while let Some(byte) = iter.next() {
            if *byte == b'\r' {
                return Ok(Resp::SimpleString(buf));
            }
            let char = char::from(*byte);
            buf.push(char);
        }
        Err(anyhow!("No CRLF found"))
    }

    pub fn parse_simple_integer(input: Bytes) -> anyhow::Result<Resp> {
        let mut result = 0;
        let mut negative = false;

        if let Some(first) = input.first() {
            if first != &INTEGER {
                return Err(anyhow!("Not a correct integer byte"));
            }
        }

        let mut iter = input.iter().skip(1).peekable();

        if **iter.peek().unwrap() == b'-' {
            negative = true;
        }

        while let Some(byte) = iter.next() {
            if *byte == b'\r' {
                if negative {
                    result *= -1;
                }
                return Ok(Resp::Integer(result));
            }
            if byte.is_ascii_digit() {
                result = result * 10 + (byte - b'0') as i64;
            } else {
                return Err(anyhow!("Incorrect digits"));
            }
        }
        Err(anyhow!("No CRLF found"))
    }

    pub fn parse_bulk_string(input: Bytes) -> anyhow::Result<Resp> {
        let mut len = 0;

        if let Some(first) = input.first() {
            if first != &BULK_STRING {
                return Err(anyhow!("Not a correct bulk string byte"));
            }
        }

        let mut iter = input.iter().skip(1).peekable();

        while let Some(byte) = iter.next() {
            if *byte == b'\r' {
                break;
            }
            len = len * 10 + (byte - b'0') as i64;
        }
        let mut iter = iter.skip(1);

        let mut data = Vec::with_capacity(len as usize);
        for _ in 0..len {
            if let Some(byte) = iter.next() {
                data.push(*byte);
            }
        }
        Ok(Resp::BulkString(Bytes::from(data)))
    }

    pub fn parse_array(input: Bytes) -> anyhow::Result<Resp> {
        // let mut len = 0;
        //
        // if let Some(first) = input.first() {
        //     if first != &ARRAY {
        //         return Err(anyhow!("Not a correct array byte"));
        //     }
        // }
        //
        // let mut iter = input.iter().skip(1).peekable();
        //
        // while let Some(byte) = iter.next() {
        //     if *byte == b'\r' {
        //         break;
        //     }
        //     len = len * 10 + (byte - b'0') as i64;
        // }
        //
        // let mut iter = iter.skip(1);
        // let mut result = Vec::with_capacity(len as usize);
        //
        // for _ in 0..len {
        //     let collected: Vec<u8> = iter.collect();
        //     let remaining = Bytes::from(collected);
        //     result.push(Resp::parse(remaining).unwrap());
        // }
        // Ok(Resp::Array(result))
        todo!()
    }

    // implement parse
    pub fn parse(input: Bytes) -> anyhow::Result<Resp> {
        let mut iter = input.iter().peekable();
        if let Some(byte) = iter.peek() {
            match byte {
                b'+' => {
                    iter.next();
                    Resp::parse_simple_string(input)
                }
                b'-' => {
                    iter.next();
                    Resp::parse_simple_string(input)
                }
                b':' => {
                    iter.next();
                    Resp::parse_simple_integer(input)
                }
                b'$' => {
                    iter.next();
                    Resp::parse_bulk_string(input)
                }
                b'*' => {
                    iter.next();
                    Resp::parse_array(input)
                }
                _ => Err(anyhow!("Invalid byte")),
            }
        } else {
            Err(anyhow!("Invalid byte"))
        }
    }
}
