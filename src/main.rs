use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
//use std::sync::{Arc, Mutex};
use std::env;
use std::convert::AsRef;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};

fn read_stream(stream: UnixStream, tx: Sender<String>) {
    let buffer_reader = BufReader::new(stream);
    for line in buffer_reader.lines() {
        let data_to_send = line.unwrap();
        println!("Recieved: {}", data_to_send);
        tx.send(data_to_send).unwrap();
    }

    println!("buffer exit");
}

fn read_stdin (tx: Sender<String>) {

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
                tx.send(input).unwrap();
            }
            Err(err) => {
                    println!("Error: {}", err);
                    break;
            }
        }
    }
    println!("Exit reading consiole input");
}

fn write_stream(rx: Receiver<String>, 
                target_stream: UnixStream) {

    let mut buffer_writer = BufWriter::new(target_stream);

    loop {
        let message = rx.recv().unwrap();
        println!("Send new message = {}", message);
        buffer_writer.write(message .as_bytes()).unwrap();
        buffer_writer.flush().unwrap();
    }
}


enum EndPointType {
    None,
    UdsServer,
    UdsClient,
    Stdio
}

struct EndPointConfiguration {
    end_point_type: EndPointType,
    address: String
}

struct SendTargetChannel{
    tx: std::sync::mpsc::Sender<String>,
    id: u8
}

const ID_ZIPGATEWAY: u8 = 10;
const ID_HUBCORE: u8 = 12;
const ID_STDIO: u8 = 14;



fn main() {
    let args: Vec<String> = env::args().collect();

    let mut zipgateway_client = EndPointConfiguration{ end_point_type: EndPointType::None, address: "".to_string()};
    let mut hub_core_server = EndPointConfiguration{ end_point_type: EndPointType::None, address: "".to_string()};

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
            EndPointType::UdsClient => {
                zipgateway_client = EndPointConfiguration{
                    address: args[i+1].clone(),
                    end_point_type: end_point_type
                }
            }
            EndPointType::UdsServer => {
                    hub_core_server = EndPointConfiguration{
                    address: args[i+1].clone(),
                    end_point_type: end_point_type
                }
            }
            _ => {

            }
        }

    }

    let (tx_zipgateway, rx_zipgateway) = channel();
    let (tx_hubcore, rx_hubcore) = channel();
    let (tx_stdio, rx_stdio) = channel();
    let (tx_brocker, rx_brocker) = channel();

    // init brocker TX channels
    let mut brocker_tx_channels = Vec::<SendTargetChannel>::new();
    brocker_tx_channels.push( SendTargetChannel{id: ID_ZIPGATEWAY, tx: tx_zipgateway.clone()});
    brocker_tx_channels.push( SendTargetChannel{id: ID_HUBCORE, tx: tx_hubcore.clone()});
    brocker_tx_channels.push( SendTargetChannel{id: ID_STDIO, tx: tx_stdio.clone()});

        // read user input
    let tx_brocker_clone_1 = tx_brocker.clone();
    let user_input_thread = thread::spawn(move || {
        read_stdin(tx_brocker_clone_1);
    });


    // brocker
    thread::spawn(move|| {
        loop {
            let recieved = rx_brocker.recv().unwrap();
            println!("Recieved: {}", recieved);
            for target in &brocker_tx_channels {
                target.tx.send(recieved.clone()).unwrap();
            }
        }
    });

     // Zip Gateway end point
    match zipgateway_client.end_point_type
    {
        EndPointType::UdsClient => {
            match UnixStream::connect(zipgateway_client.address) {
                Ok(stream) => {
                    let tx_brocker_clone_2 = tx_brocker.clone();
                    let stream_clone = stream.try_clone().unwrap();
                    thread::spawn(move || write_stream(rx_zipgateway, stream_clone));
                    thread::spawn(move || read_stream(stream, tx_brocker_clone_2));
                }
                Err(err) => {
                    println!("Error: {}", err);
                }
            }
        }
        _ => {
            // noop
        }
    }

    // will block the thread
    // should be moved from main thread
    match hub_core_server.end_point_type {
        EndPointType::UdsServer => {
            let listener = UnixListener::bind(hub_core_server.address).unwrap();
            for stream in listener.incoming() {
                match stream {
                    // Hub core end point
                    Ok(stream) => {
                        println!("New incoming stream.");

                        let tx_brocker_clone_3 = tx_brocker.clone();

                        // TODO: 
                        // let stream1 = stream.try_clone().unwrap();
                        // thread::spawn(move || write_stream(rx_hubcore, stream1));

                        thread::spawn(move || read_stream(stream, tx_brocker_clone_3));
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
        }
        _ => {
            // no op
        }
    }

    user_input_thread.join().expect("user_input_thread paniced");
}
