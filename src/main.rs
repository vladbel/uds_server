use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use std::sync::{Arc, Mutex};


fn read_from_client(id: u8, stream: UnixStream) {
    let buffer_reader = BufReader::new(stream);
    for line in buffer_reader.lines() {
        println!("From {}: {}", id, line.unwrap());
    }

    println!("buffer exit");
}

fn read_stdin (arc_mutex: Arc<Mutex<String>>) {

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
                let mut message = arc_mutex.lock().unwrap();
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

fn send_to_client(id: u8,
                  target_stream: UnixStream,
                  arc_mutex: Arc<Mutex<String>>) {
    
    let mut buffer_writer = BufWriter::new(target_stream);
    let mut handled_message = String::new();

    loop {
        std::thread::sleep(std::time::Duration::new(2, 0));
        let local_message = arc_mutex.lock().unwrap();
        if *local_message != handled_message {
            handled_message = local_message.clone();
            println!("Send to {} new message = {}", id, local_message);
            buffer_writer.write(handled_message .as_bytes()).unwrap();
            buffer_writer.flush().unwrap();
            if handled_message == "exit\n" {
                break;
            }
        }
    }

    println!("Exit reading consiole input");
}

fn main() {
    let listener = UnixListener::bind("/tmp/rust_ipc_socket").unwrap();

    let arc_message = Arc::new(Mutex::new(<String>::new()));

     
    let arc_message_ref = Arc::clone(&arc_message);
    thread::spawn(move || {
        read_stdin( arc_message_ref );
    });

    let arc_message_ref_2 =Arc::clone(&arc_message);

    thread::spawn(move || {
        let mut handled_message = String::new();
        loop {
            std::thread::sleep(std::time::Duration::new(2, 0));
            //println!("Read shared resource: trying to aquire Mutex on messge");
            let local_message = arc_message_ref_2.lock().unwrap();
            if *local_message != handled_message {
                handled_message = local_message.clone();
                println!("new message = {}", local_message);
            }
        }
    });

    let mut client_num: u8 = 0;
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                client_num = client_num + 1;
                println!("New incoming stream");
                let stream1 = stream.try_clone().unwrap();
                let arc_message_ref_3 =Arc::clone(&arc_message);
                thread::spawn(move || send_to_client(client_num, stream1, arc_message_ref_3));

                thread::spawn(move || read_from_client(client_num, stream));

            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}
