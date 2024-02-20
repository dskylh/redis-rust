use std::{io::Write, net::TcpStream};

pub fn ping(stream: &mut TcpStream) {
    let buf = b"+PONG\r\n";
    let _ = stream.write_all(buf);
}
