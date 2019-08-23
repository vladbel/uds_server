use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
use std::sync::{Arc, Mutex};
use std::env;
use std::convert::AsRef;
use std::sync::mpsc::channel;
use serde_json::{Result, Value};


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
    None,
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
        let mut end_point_type: EndPointType = EndPointType::None;
        match arg.as_ref() {
            "--uds_client" => {
                end_point_type = EndPointType::UdsClient;
            }
            "--uds_server" => {
                end_point_type = EndPointType::UdsServer;
            }
            _ => {
                // Noop
            }
        }

        match end_point_type {
            EndPointType::None => {
                // Noop
            }
            _ => {
                if i + 1 < args.len() {
                    let copy = args[i + 1].clone();
                    let v: Value = serde_json::from_str(&copy).unwrap();
                    let address = v["address"].to_string();
                    //let targets = &v["targets"];
                    let id = v["id"].to_string();
                    end_points.push ( EndPointConfiguration{end_point_type: end_point_type,
                                                            address: address});
                }
            }
        }


    }

    return;
    // init cross-thread message bus
    // currently just a string

    let (tx, rx) = channel();

    let tx1 = tx.clone();
    thread::spawn(move|| {
        loop{
            std::thread::sleep(std::time::Duration::new(1, 0));
            tx1.send("blah").unwrap();
        }
    });

    let tx2 = tx.clone();
    thread::spawn(move|| {
        loop {
            std::thread::sleep(std::time::Duration::new(1, 0));
            tx2.send("blah-blah").unwrap();
        }

    });

    thread::spawn(move|| {
        loop {
            std::thread::sleep(std::time::Duration::new(1, 0));
            tx.send("blaha-muha-ha-ha-ha").unwrap();
        }
        
    });

    thread::spawn(move|| {
        loop {
            let recieved = rx.recv().unwrap();
            println!("Recieved: {}", recieved);
        }

    });




    let arc_message = Arc::new(Mutex::new(<String>::new()));
    let arc_message_ref = Arc::clone(&arc_message);

    // read user input
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
                    // will block the thread
                    // should be moved from main thread
                    let listener = UnixListener::bind(end_point.address).unwrap();
                    for stream in listener.incoming() {
                        match stream {
                            Ok(stream) => {
                                client_num = client_num + 1;
                                println!("New incoming stream.");
                                let stream1 = stream.try_clone().unwrap();
                                let arc_message_ref_3 =Arc::clone(&arc_message);
                                thread::spawn(move || write_stream(client_num, stream1, arc_message_ref_3));

                                thread::spawn(move || read_stream(client_num, stream));

                            }
                            Err(err) => {
                                println!("Error: {}", err);
                                break;
                            }
                        }
                    }
            }
            _ => {
                // NoOp
            }
        }
    }

    user_input_thread.join().expect("user_input_thread paniced");
}
