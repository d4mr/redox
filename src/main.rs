use std::str;

use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

enum Command {
    Ping,
}

const INTERFACE: &str = "127.0.0.1";
const PORT: &str = "6379";

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Starting Server on {} at port {}", INTERFACE, PORT);
    let listener = TcpListener::bind(format!("{}:{}", INTERFACE, PORT))
        .await
        .unwrap();

    println!("Listening at {}", listener.local_addr().unwrap());

    loop {
        let (stream, _) = listener.accept().await.unwrap();
        process(stream).await;
    }
}

async fn process(mut stream: TcpStream) {
    handle_command(Command::Ping, &mut stream).await;
}

async fn handle_command(command: Command, stream: &mut TcpStream) {
    match command {
        Command::Ping => {
            stream
                .write_all("+PONG\r\n".as_bytes())
                .await
                .expect("could not write to buffer");
            println!("Responding with: pong");
        }
    }
}
