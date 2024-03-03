use std::{
    io::{Read, Write},
    net::TcpListener,
    str,
};

const INTERFACE: &str = "127.0.0.1";
const PORT: &str = "6379";

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Starting Server on {} at port {}", INTERFACE, PORT);

    let listener = TcpListener::bind(format!("{}:{}", INTERFACE, PORT)).unwrap();

    println!("Listening at {}", listener.local_addr().unwrap());

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = [0; 1024];

                println!("accepted new connection");
                loop {
                    let n = stream.read(&mut buf).expect("Could not read from stream");
                    if n == 0 {
                        continue;
                    }

                    let s = match str::from_utf8(&buf) {
                        Ok(v) => v,
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };

                    stream
                        .write_all("+PONG\r\n".as_bytes())
                        .expect("could not write to buffer");

                    println!("Got message from connection: {}", s);
                    println!("Responding with: pong");
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
