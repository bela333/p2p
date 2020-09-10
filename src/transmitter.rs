use std::{path::Path, fs::File};

use crate::{networking::NetworkHandler, message::Messages, message::FileTransferRequestMessage};

pub async fn begin(handler: NetworkHandler, path: &Path){
    let file = File::open(path).unwrap();
    handler.wait_for_connection().await;
    println!("Connected!");
    let mut receiver = handler.subscribe();

    {
        let msg = FileTransferRequestMessage{
            filename: path.file_name().unwrap().to_str().unwrap().to_string(),
            filesize: file.metadata().unwrap().len()
        };
        handler.get_sender().send_reliable(Messages::FileTransferRequest(msg)).await.unwrap();
    }

    while let Ok(msg) = receiver.recv().await {
        
    }
}