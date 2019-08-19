use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;


fn read_client(stream: UnixStream) {
    let buffer_reader = BufReader::new(stream);
    for line in buffer_reader.lines() {
        println!("{}", line.unwrap());
    }

    println!("buffer exit");
}

fn send_to_clients(stream: UnixStream) {
    let mut input = String::new();
    let mut input_length = 0;
    let mut buffer_writer = BufWriter::new(stream);
    while input_length < 3 {
        match std::io::stdin().read_line(&mut input) {
            Ok(input) => {
                println!("Input: {}", input);
                input_length = input_length + 1;
                buffer_writer.write(b"from server to clients").unwrap();
                buffer_writer.flush().unwrap();
            }
            Err(err) => {
                    println!("Error: {}", err);
                    break;
            }
        }
    }
}

fn main() {
    let listener = UnixListener::bind("/tmp/rust_ipc_socket").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New incoming stream");
                let stream1 = stream.try_clone().unwrap();

                thread::spawn(|| send_to_clients(stream1));
                thread::spawn(|| read_client(stream));

            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}
