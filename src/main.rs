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

fn send_to_clients(target_stream: UnixStream) {
    
    let mut buffer_writer = BufWriter::new(target_stream);
    let mut read_input = true;
    while read_input {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(input_length) => {
                println!("Input: {}", input);
                println!("Input length: {}", input_length);
                if input == "exit" {
                    read_input = false;
                }
                buffer_writer.write(input.as_bytes()).unwrap();
                buffer_writer.flush().unwrap();
            }
            Err(err) => {
                    println!("Error: {}", err);
                    break;
            }
        }
    }
    println!("Exit reading consiole input");
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
