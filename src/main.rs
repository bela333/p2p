
extern crate stunclient;
extern crate tokio;
extern crate clap;

pub mod obfuscator;
pub mod error;
pub mod bitfield;
pub mod message;
pub mod networking;
mod receiver;
mod transmitter;
mod test;

use clap::{App, Arg};
use tokio::io::BufReader;
use tokio::time::{Duration};
use tokio::io::AsyncBufReadExt;
use obfuscator::AddressInfo;
use std::{io::Write, net::{SocketAddr}, path::Path, net::IpAddr};
use clipboard::{ClipboardContext, ClipboardProvider};

const PING_INTERVAL: u64 = 10;

#[tokio::main]
pub async fn main(){
    //Determine role
    let matches = App::new("Peer-to-peer file sender")
        .version("0.0.1")
        .about("Easy file sending")
        .arg(Arg::with_name("FILE")
            .index(1))
        .get_matches();
    let path = matches.value_of("FILE");
    let path = path.map(|path|Path::new(path));
    let path = {
        if let Some(path) = path{
            if !path.exists() || path.is_dir(){
                None
            }else{
                Some(path)
            }
        }else{
            path
        }
    };


    let (socket, local_address) = stunclient::just_give_me_the_udp_socket_and_its_external_address();
    //let local_address = SocketAddr::new(IpAddr::V4("127.0.0.1".parse().unwrap()), socket.local_addr().unwrap().port());
    //Convert address to IPv4 address
    let local_addressv4 = {
        if let SocketAddr::V4(address) = local_address{
            address
        }else{
            panic!("Invalid IP type");
        }
    };
    
    {
        if let Some(path) = path{
            println!("You are about to transmit {}", path.file_name().unwrap().to_str().unwrap());
        }
        let info = AddressInfo::new(local_addressv4);
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        ctx.set_contents(info.to_string()).unwrap();
        println!("Your code is: {} (copied to clipboard)", info); //Print sync code
    }

    //Setup an interval for the ping messages
    let mut ping_interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL));
    //Setup a stream of lines from stdin
    let mut line_stream = BufReader::new(tokio::io::stdin()).lines();
    //Constantly ping, until a new line comes
    print!("Partner's code: ");
    std::io::stdout().flush().unwrap();
    let remote: AddressInfo = loop {
        tokio::select! {
            line = line_stream.next_line() => {
                let line = line.unwrap().unwrap();
                if let Ok(info) = line.parse::<AddressInfo>(){
                    break info;
                }else{
                    println!("Invalid code");
                    print!("Partner's code: ");
                    std::io::stdout().flush().unwrap();
                }
            }
            _ = ping_interval.tick() => {
                socket.send_to("PING".as_bytes(), "stun.l.google.com:19302").unwrap();
            }
        }
    };
    println!("{}", remote.address);
    let socket_addr = socket.local_addr().unwrap();
    drop(socket);
    let network_handler = networking::NetworkHandler::new(socket_addr, SocketAddr::V4(remote.address));
    network_handler.begin().await.unwrap();
    if let Some(path) = path{
        //Transmitting
        transmitter::begin(network_handler, path).await;
    }else{
        //Receiving
        receiver::begin(network_handler).await;
    }

}