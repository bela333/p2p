use crate::{networking::NetworkHandler, message::Messages};

pub async fn begin(handler: NetworkHandler){
    println!("Connected!");
    let mut receiver = handler.subscribe();
    while let Ok(msg) = receiver.recv().await {
        if let Messages::FileTransferRequest(msg) = msg{
            println!("Receiving {} which is {} bytes", msg.filename, msg.filesize);
        }
    }
}