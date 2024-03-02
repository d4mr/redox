use std::net::TcpListener;

const INTERFACE: &str = "127.0.0.1";
const PORT: &str = "6379";

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Listening on {} at port {}", INTERFACE, PORT);

    let listener = TcpListener::bind(format!("{}:{}", INTERFACE, PORT)).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
