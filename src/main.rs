use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixStream};
use std::thread;
use std::sync::{Arc, Mutex};
use std::env;
use std::convert::AsRef;


fn read_stream(id: u8, stream: UnixStream) {
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

fn write_stream(id: u8,
                  target_stream: UnixStream,
                  arc_mutex: Arc<Mutex<String>>) {

    let mut buffer_writer = BufWriter::new(target_stream);
    let mut handled_message = String::new();

    loop {
        std::thread::sleep(std::time::Duration::new(1, 0));
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


enum EndPointType {
    UdsServer,
    UdsClient
}
struct EndPointConfiguration {
    end_point_type: EndPointType,
    address: String
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut end_points = Vec::<EndPointConfiguration>::new();

    for (i, arg) in args.iter().enumerate() {
        match arg.as_ref() {
            "--uds_client" => {
                if i + 1 < args.len() {
                    let copy = args[i + 1].clone();
                    end_points.push ( EndPointConfiguration{end_point_type: EndPointType::UdsClient,
                                                        address: copy});
                }
            }
            "--uds_server" => {
                if i + 1 < args.len() {
                    let copy = args[i + 1].clone();
                    end_points.push ( EndPointConfiguration{end_point_type: EndPointType::UdsServer,
                                                        address: copy});
                }
            }
            _ => {
                // Noop
            }
        }
    }

    let arc_message = Arc::new(Mutex::new(<String>::new()));
    let arc_message_ref = Arc::clone(&arc_message);

    let user_input_thread = thread::spawn(move || {
        read_stdin( arc_message_ref );
    });

    let mut client_num: u8 = 0;

    for end_point in end_points {
        match end_point.end_point_type {
            EndPointType::UdsClient => {
                match UnixStream::connect(end_point.address) {
                    Ok(stream) => {
                        client_num = client_num + 1;
                        let stream1 = stream.try_clone().unwrap();
                        let arc_message_ref_3 =Arc::clone(&arc_message);
                        thread::spawn(move || write_stream(client_num, stream1, arc_message_ref_3));
                        thread::spawn(move || read_stream(client_num, stream));
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
            EndPointType::UdsServer => {
            }
        }
    }

    user_input_thread.join().expect("user_input_thread paniced");
}
