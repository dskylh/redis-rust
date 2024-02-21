// Uncomment this block to pass the first stage
use redis_starter_rust::ping::ping;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;
        tokio::spawn(async move { ping(&mut socket).await });
    }
}

// make tests for parser.rs
#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use redis_starter_rust::parser::*;

    #[test]
    fn test_parse_simple_string() {
        let input = Bytes::from_static(b"+OK\r\n");
        let result = Resp::parse_simple_string(input).unwrap();
        assert_eq!(result, (Resp::SimpleString("OK".to_string()), input.len()));
    }

    #[test]
    fn test_parse_simple_integer() {
        let input = Bytes::from_static(b":1000\r\n");
        let result = Resp::parse_simple_integer(input).unwrap();
        assert_eq!(result, Resp::Integer(1000));
    }

    #[test]
    fn test_parse_bulk_string() {
        let input = Bytes::from_static(b"$6\r\nfoobar\r\n");
        let result = Resp::parse_bulk_string(input).unwrap();
        assert_eq!(result, Resp::BulkString(Bytes::from("foobar")));
    }
    // test the array
    #[test]
    fn test_parse_array() {
        let input = Bytes::from_static(b"*2\r\n+OK\r\n:1000\r\n");
        let result = Resp::parse_array(input).unwrap();
        assert_eq!(
            result,
            Resp::Array(vec![
                Resp::SimpleString("OK".to_string()),
                Resp::Integer(1000)
            ])
        );
    }
}
