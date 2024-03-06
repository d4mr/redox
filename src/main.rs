use bytes::{Bytes, BytesMut};

use resp::RespConcreteType;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use std::{collections::HashMap, sync::Arc};

enum Command {
    Ping,
    Set(String, String),
    Get(String),
    Echo(String),
    Error(Bytes),
}

enum CommandError {
    UnknownCommand(String),
    BadLength(usize),
}

type Storage = Arc<Mutex<HashMap<String, String>>>;

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

    let storage: Storage = Arc::new(Mutex::new(HashMap::new()));

    loop {
        let (stream, addr) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new ta sk and processed there.
        println!("New Connection at {addr}");
        let st = storage.clone();
        tokio::spawn(async {
            process(stream, st).await;
        });
    }
}

async fn process(mut stream: TcpStream, storage: Arc<Mutex<HashMap<String, String>>>) {
    let mut buf = BytesMut::with_capacity(20);
    let mut partial: Option<resp::RespTypePartialable> = None;

    loop {
        let st = storage.clone();
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
                handle_command(command, &mut stream, st).await;
            }
        }
    }
}

fn parse_command(res: RespConcreteType) -> Result<Command, CommandError> {
    match res {
        RespConcreteType::Array(mut array) => match &array[0] {
            RespConcreteType::BulkString(command) => match command.to_lowercase().as_str() {
                "ping" => Ok(Command::Ping),
                "echo" => match &array[1] {
                    RespConcreteType::BulkString(arg) => Ok(Command::Echo(arg.to_string())),
                    _ => Err(CommandError::UnknownCommand(
                        "Invalid Echo Command".to_string(),
                    )),
                },
                "set" => {
                    if array.len() != 3 {
                        return Err(CommandError::BadLength(array.len()));
                    }

                    let value = array.pop();
                    let key = array.pop();

                    match (key, value) {
                        (
                            Some(RespConcreteType::BulkString(key)),
                            Some(RespConcreteType::BulkString(value)),
                        ) => Ok(Command::Set(key, value)),
                        _ => Err(CommandError::UnknownCommand(
                            "Invalid Set Command".to_string(),
                        )),
                    }
                }
                "get" => {
                    if array.len() != 2 {
                        return Err(CommandError::BadLength(array.len()));
                    }

                    let key = array.pop();

                    match key {
                        Some(RespConcreteType::BulkString(key)) => Ok(Command::Get(key)),
                        _ => Err(CommandError::UnknownCommand(
                            "Invalid Get Command".to_string(),
                        )),
                    }
                }
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

async fn handle_command(
    command: Command,
    stream: &mut TcpStream,
    storage: Arc<Mutex<HashMap<String, String>>>,
) {
    match command {
        Command::Ping => {
            stream
                .write_all("+PONG\r\n".as_bytes())
                .await
                .expect("could not write to buffer");
        }
        Command::Echo(arg) => {
            stream
                .write_all(format!("${}\r\n{arg}\r\n", arg.len()).as_bytes())
                .await
                .expect("could not write to buffer");
        }
        Command::Error(arg) => {
            stream
                .write_all(&arg)
                .await
                .expect("could not write to buffer");
        }
        Command::Set(key, value) => {
            let mut storage = storage.lock().await;
            storage.insert(key, value);
            stream
                .write_all("+OK\r\n".as_bytes())
                .await
                .expect("could not write to buffer");
        }
        Command::Get(key) => {
            let storage = storage.lock().await;
            let value = storage.get(&key);
            match value {
                Some(value) => {
                    stream
                        .write_all(format!("${}\r\n{}\r\n", value.len(), value).as_bytes())
                        .await
                        .expect("could not write to buffer");
                }
                None => {
                    stream
                        .write_all("$-1\r\n".as_bytes())
                        .await
                        .expect("could not write to buffer");
                }
            }
        }
    };
}
