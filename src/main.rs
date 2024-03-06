use std::{fmt::format, str};

use bytes::{BufMut, Bytes, BytesMut};

use resp::RespConcreteType;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

enum Command {
    Ping,
    Echo(String),
    Error(Bytes),
}

enum CommandError {
    UnknownCommand(String),
}

const INTERFACE: &str = "127.0.0.1";
const PORT: &str = "6379";

mod resp;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Starting Server on {} at port {}", INTERFACE, PORT);
    let listener = TcpListener::bind(format!("{}:{}", INTERFACE, PORT))
        .await
        .unwrap();

    println!("Listening at {}", listener.local_addr().unwrap());

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new ta sk and processed there.
        println!("New Connection at {addr}");
        tokio::spawn(async move {
            process(stream).await;
        });
    }
}

async fn process(mut stream: TcpStream) {
    let mut buf = BytesMut::with_capacity(20);
    let mut partial: Option<resp::RespTypePartialable> = None;

    loop {
        let s = stream
            .read_buf(&mut buf)
            .await
            .expect("Could not read message");

        if s == 0 {
            continue;
        }

        let parse_result = resp::parse(&mut buf, partial).expect("Unexpected parsing failure");

        match parse_result {
            resp::Resp::Partial(partial_res) => {
                partial = Some(partial_res);
                continue;
            }
            resp::Resp::Concrete(res) => {
                partial = None;
                let command = parse_command(res)
                    .unwrap_or_else(|_| Command::Error(Bytes::from("+Invalid Command\r\n")));
                handle_command(command, &mut stream).await;
            }
        }
    }
}

fn parse_command(res: RespConcreteType) -> Result<Command, CommandError> {
    match res {
        RespConcreteType::Array(array) => match &array[0] {
            RespConcreteType::String(command) => match command.to_lowercase().as_str() {
                "ping" => Ok(Command::Ping),
                "echo" => match &array[1] {
                    RespConcreteType::String(arg) => Ok(Command::Echo(arg.to_string())),
                    _ => Err(CommandError::UnknownCommand(
                        "Invalid Echo Command".to_string(),
                    )),
                },
                _ => Err(CommandError::UnknownCommand(command.to_string())),
            },
            _ => panic!("Unknown command"),
        },
        _ => panic!("Unknown command"),
    }
}

// async fn process_all(
//     mut buf: &mut BytesMut,
//     mut stream: TcpStream,
//     partial: Option<resp::RespTypePartialable>,
// ) -> Resp {
//     let s = stream
//         .read_buf(&mut buf)
//         .await
//         .expect("Could not read message");

//     if s == 0 {
//         return process_all(buf, stream, partial).await;
//     }

//     let parse_result = resp::parse(&mut buf, partial).expect("damn");

//     match parse_result {
//         resp::Resp::Partial(partial_res) => {
//             println!("Partial: {:?}", partial_res);
//             return process_all(buf, stream, Some(partial_res)).await;
//         }
//         concrete_type => concrete_type,
//     }
// }

async fn handle_command(command: Command, stream: &mut TcpStream) {
    match command {
        Command::Ping => {
            stream
                .write_all("+PONG\r\n".as_bytes())
                .await
                .expect("could not write to buffer");
            println!("Responding with: pong");
        }
        Command::Echo(arg) => {
            stream
                .write_all(format!("${}\r\n{arg}\r\n", arg.len().to_string()).as_bytes())
                .await
                .expect("could not write to buffer");
            println!("Responding with: pong");
        }
        Command::Error(arg) => {
            stream
                .write_all(&arg)
                .await
                .expect("could not write to buffer");
        }
    }
}
