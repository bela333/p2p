
extern crate stunclient;
extern crate tokio;

pub mod obfuscator;
pub mod error;
pub mod bitfield;
pub mod message;
pub mod networking;
mod test;

use tokio::io::BufReader;
use tokio::time::{Duration};
use tokio::io::AsyncBufReadExt;
use obfuscator::AddressInfo;
use std::net::SocketAddr;
use clipboard::{ClipboardContext, ClipboardProvider};

const PING_INTERVAL: u64 = 10;

#[tokio::main]
pub async fn main(){
    let (socket, local_address) = stunclient::just_give_me_the_udp_socket_and_its_external_address();

    //Convert address to IPv4 address
    let local_addressv4 = {
        if let SocketAddr::V4(address) = local_address{
            address
        }else{
            panic!("Invalid IP type");
        }
    };
    
    {
        let info = AddressInfo::new(local_addressv4);
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        ctx.set_contents(info.to_string()).unwrap();
        println!("Your code is: {} (copied to clipboard)", info); //Print sync code
    }

    //Setup an interval for the ping messages
    let mut ping_interval = tokio::time::interval(Duration::from_secs(PING_INTERVAL));
    //Setup a stream of lines from stdin
    let mut line_stream = BufReader::new(tokio::io::stdin()).lines();
    let mut code: Option<String> = None;
    //Constantly ping, until a new line comes
    loop {
        tokio::select! {
            line = line_stream.next_line() => {
                let line = line.unwrap().unwrap();
                code = Some(line);
                break;
            }
            _ = ping_interval.tick() => {
                socket.send_to("PING".as_bytes(), "stun.l.google.com:19302").unwrap();
            }
        }
    }
    let remote: AddressInfo = code.unwrap().parse().unwrap();
    println!("{}", remote.address);
    let mut network_handler = networking::NetworkHandler::new(local_address, SocketAddr::V4(remote.address));
    network_handler.begin().await.unwrap();

}