use std::io::{BufRead, BufReader, BufWriter};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;

fn read_client(buffer_reader: BufReader<UnixStream>) {
    for line in buffer_reader.lines() {
        println!("{}", line.unwrap());
    }

    println!("buffer exit");
}

fn send_to_client() {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input);
    //buffer_writer.write(input.unwrap());
    println!("send to client {}", input);
}

fn main() {
    let listener = UnixListener::bind("/tmp/rust_ipc_socket").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let buffer_reader = BufReader::new(stream);
                thread::spawn(|| read_client(buffer_reader));

                //let buffer_writer = BufWriter::new(stream);
                thread::spawn(|| send_to_client());
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}
