
extern crate stunclient;
extern crate tokio;

pub mod obfuscator;
pub mod error;
mod test;

use tokio::io::BufReader;
use tokio::time::{self, Duration};
use tokio::io::AsyncBufReadExt;
use obfuscator::AddressInfo;
use std::net::SocketAddr;

const PING_INTERVAL: u64 = 10;

#[tokio::main]
pub async fn main(){
    let (socket, address) = stunclient::just_give_me_the_udp_socket_and_its_external_address();

    //Convert address to IPv4 address
    let addressv4 = {
        if let SocketAddr::V4(address) = address{
            address
        }else{
            panic!("Invalid IP type");
        }
    };
    
    {
        let info = AddressInfo::new(addressv4);
        println!("Your code is: {}", info); //Print sync code
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
    let info: AddressInfo = code.unwrap().parse().unwrap();
    println!("{}", info.address);
}