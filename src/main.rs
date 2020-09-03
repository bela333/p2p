extern crate stunclient;

pub mod obfuscator;
pub mod error;
mod test;

#[tokio::main]
pub async fn main(){
    let (socket, address) = stunclient::just_give_me_the_udp_socket_and_its_external_address();
    println!("{}", address);
}