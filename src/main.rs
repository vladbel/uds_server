use std::io::{BufReader, BufWriter, Write, Read};
use std::os::unix::net::{UnixListener, UnixStream};
use std::thread;
//use std::sync::{Arc, Mutex};
use std::env;
use std::convert::AsRef;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};

fn read_stream(id: u8, stream: UnixStream, tx: Sender<ChannelMessage>) {
    let mut buffer_reader = BufReader::new(stream);
    loop {
        let mut buffer: [u8; 8] = [0; 8];
        match buffer_reader.read(&mut buffer) {
            Ok(bytes_received) => {
                println! ("DEBUG: read_stream(): -- {} -- bytes received.", bytes_received);

                // publish received data
                let text_to_send = format!("{} bytes recieved: {:X?}", bytes_received, &buffer[0 .. bytes_received]);
                let mut subscribers = Vec::<u8>::new();
                subscribers.push(ID_ALL);
                let message = ChannelMessage { 
                    sender_id: id, 
                    text: text_to_send , 
                    data: buffer[0 .. (bytes_received)].to_vec(), 
                    subscribers: subscribers};
                tx.send(message).unwrap();
            }
            Err(err) => {
                println! ("ERROR: read_stream(): error reading stream {}.", err);
            }
        }
    }
}

fn read_stdin (id: u8, tx: Sender<ChannelMessage>) {

    loop {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(input_length) => {
                println!("Input: {}", input);
                println!("Input length: {}", input_length);

                let mut data = Vec::<u8>::new();
                let mut subscribers = Vec::<u8>::new();
                match input.as_ref() {
                    "exit\n" => {
                        break;
                    }
                    "get_zipgateway_version\n" => {
                        data.push(0x01); 
                        subscribers.push(ID_ZIPGATEWAY);
                    }
                    _ => {
                        // noop
                        data = (&input.as_bytes()).to_vec();
                        subscribers.push(ID_ALL);
                    }
                }


                let message = ChannelMessage { sender_id: id, text: input, data: data, subscribers: subscribers};
                tx.send(message).unwrap();
            }
            Err(err) => {
                    println!("Error: {}", err);
                    break;
            }
        }
    }
    println!("Exit reading consiole input");
}

fn write_stream(rx: Receiver<ChannelMessage>, 
                target_stream: UnixStream) {

    let mut buffer_writer = BufWriter::new(target_stream);

    loop {
        let message = rx.recv().unwrap();
        buffer_writer.write(&message.data).unwrap();
        buffer_writer.flush().unwrap();
    }
}

const ID_ALL: u8 = 0;
const ID_ZIPGATEWAY: u8 = 10;
const ID_HUBCORE: u8 = 12;
const ID_STDIO: u8 = 14;

enum EndPointType {
    None,
    UdsServer,
    UdsClient
}

struct EndPointConfiguration {
    end_point_type: EndPointType,
    address: String
}

// ****************************************************************
struct ChannelMessage {
    sender_id: u8,
    subscribers: Vec<u8>,
    data: Vec::<u8>,
    text: String
}

impl ChannelMessage {
    fn clone(&self) -> ChannelMessage {
        return ChannelMessage { 
            sender_id: self.sender_id, 
            data: self.data.clone(),
            text: self.text.clone(),
            subscribers: self.subscribers.clone()
        };
    }

    /*
    fn is_subscribed( &self, subscriber_id: u8) -> bool {
        let i = self.subscribers.clone().into_iter().find(|&r| {
                (r == subscriber_id) | (r == ID_ALL)
            }).unwrap();

        return i >= 0;
    }
    */

}

//  Subscriber channel

struct SubscriberChannel {
    tx: std::sync::mpsc::Sender<ChannelMessage>,
    id: u8
}

impl SubscriberChannel {
    fn send (&self, message: ChannelMessage) {
        if self.id != message.sender_id {
            self.tx.send(message).unwrap();
        }
    }
}

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
                // noop
            }
        }

    }

    // init brocker TX channels
    let mut brocker_tx_channels = Vec::<SubscriberChannel>::new();
    let (tx_brocker, rx_brocker) = channel();

    // init hub-core
    // TODO
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


                        let (tx_hubcore, rx_hubcore) = channel();
                        brocker_tx_channels.push( SubscriberChannel{id: ID_HUBCORE, tx: tx_hubcore.clone()});

                        let stream1 = stream.try_clone().unwrap();
                        thread::spawn(move || write_stream(rx_hubcore, stream1));

                        thread::spawn(move || read_stream(ID_HUBCORE,  stream, tx_brocker_clone_3));
                        break;
                    }
                    Err(err) => {
                        println!("Error: {}", err);
                    }
                }
            }
        }
        _ => {
            // noop
        }
    }
    //

    let (tx_zipgateway, rx_zipgateway) = channel();
    let (tx_stdio, rx_stdio) = channel();

    brocker_tx_channels.push( SubscriberChannel{id: ID_ZIPGATEWAY, tx: tx_zipgateway.clone()});
    brocker_tx_channels.push( SubscriberChannel{id: ID_STDIO, tx: tx_stdio.clone()});


    // brocker
    thread::spawn(move|| {
        loop {
            let recieved = rx_brocker.recv().unwrap();
            println!("Recieved by message brocker : {}", recieved.text);

            for subscriber in &brocker_tx_channels {
                subscriber.send(recieved.clone());
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
                    thread::spawn(move || read_stream(ID_ZIPGATEWAY, stream, tx_brocker_clone_2));
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

    // Init stdio
    // std output
    thread::spawn(move || {  // read standard input and send it to brocker channel
        loop {
            let message = rx_stdio.recv().unwrap();
            match message.sender_id {
                ID_HUBCORE => {
                    println! (" Recieved from HUB:\ntext:  {} \n data: {:X?}", message.text, message.data );
                }
                ID_ZIPGATEWAY => {
                    println! ("Recieved from ZIPGATEWAY:\ntext:  {} \n data: {:X?}", message.text, message.data );
                }
                ID_STDIO => {
                    println! (" This should never happen: sent from STDIO to STDIO.");
                }
                _ => {
                    println! (" This should never happen: unknown sender ");
                }
            }
            println! ()
        }
    });


    // read user input
    let tx_brocker_clone_1 = tx_brocker.clone();
    let std_input_thread = thread::spawn(move || {  // read standard input and send it to brocker channel
        read_stdin( ID_STDIO,  tx_brocker_clone_1);
    });

    std_input_thread.join().expect("user_input_thread paniced");
}
