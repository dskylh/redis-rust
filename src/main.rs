// Uncomment this block to pass the first stage
use std::net::TcpListener;

use redis_starter_rust::ping::ping;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    loop {
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => ping(&mut stream),
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        }
    }
}
