use futures::Stream;
use tokio::pin;

use crate::{networking::NetworkHandler, message::Messages, message::FileTransferAcceptMessage};

pub async fn begin(handler: NetworkHandler){
    println!("Connected!");
    let mut receiver = handler.subscribe();
    let mut sender = handler.get_sender();

    loop{
        if let Ok(msg) = receiver.recv().await {
            match msg {
                Messages::FileTransferRequest(msg) => {
                    println!("Receiving {} which is {} bytes", msg.filename, msg.filesize);
                    let msg = Messages::FileTransferAccept(FileTransferAcceptMessage{});
                    sender.send_reliable(msg).await.unwrap();      
                },
                Messages::PartBegin(msg) =>{
                    println!("Beginning part {}, which is {} byte", msg.part_number, msg.part_size)
                },
                Messages::Chunk(msg) => {
                    println!("Incoming chunk #{}. Part of part {} and is {} bytes", msg.index, msg.part_number, msg.data.len())
                },
                Messages::PartEnd(msg) => {
                    println!("Finished receiving file");

                }
                _ => continue
            }
        }
    }
}