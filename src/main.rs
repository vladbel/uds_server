use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use std::sync::{Arc, Mutex};


fn read_from_client(stream: UnixStream) {
    let buffer_reader = BufReader::new(stream);
    for line in buffer_reader.lines() {
        println!("{}", line.unwrap());
    }

    println!("buffer exit");
}

fn read_stdin (message: &mut std::string::String) {

    let mut read_input = true;
    while read_input {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(input_length) => {
                println!("Input: {}", input);
                println!("Input length: {}", input_length);
                if input == "exit\n" {
                    read_input = false;
                }
                *message = input;
            }
            Err(err) => {
                    println!("Error: {}", err);
                    break;
            }
        }
    }
    println!("Exit reading consiole input");
}

fn send_to_client(target_stream: UnixStream) {
    
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
    
    let arc_message = Arc::new(Mutex::new(<String>::new()));

     
    let arc_message_ref =Arc::clone(&arc_message);
    thread::spawn(move || {
        let mut local_message = arc_message_ref.lock().unwrap();
        read_stdin( &mut local_message);
    });

    let arc_message_ref_2 =Arc::clone(&arc_message);
    thread::spawn(move || {
        println!("Trying to aquire Mutex on messge");
        let local_message = arc_message_ref_2.lock().unwrap();
        println!("Message = {}", local_message);
    });


    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New incoming stream");
                let stream1 = stream.try_clone().unwrap();

                thread::spawn(move || send_to_client(stream1));

                thread::spawn(move || read_from_client(stream));

            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}
