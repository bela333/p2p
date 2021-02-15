use std::{
    fs::{self, File},
    io::Write,
    iter::successors,
    path::{self, Path},
};

use bytes::BytesMut;
use futures::{FutureExt, Stream};
use tokio::{pin, sync::broadcast::Receiver};

use crate::{
    bitfield::Bitfield,
    message::FileTransferAcceptMessage,
    message::{Messages, PartBeginMessage, TransferIncompleteMessage, TransferSuccessfulMessage},
    networking::NetworkHandler,
};

pub async fn begin(handler: NetworkHandler) {
    let dir = Path::new("downloads/");
    fs::create_dir_all(dir).unwrap();
    println!("Ready for transmission");
    let mut receiver = handler.subscribe();
    let mut sender = handler.get_sender();

    while let Ok(msg) = receiver.recv().await {
        if let Messages::FileTransferRequest(msg) = msg {
            println!("Downloading {} byte file: {}", msg.filesize, msg.filename);
            let mut file = fs::File::create(dir.join(msg.filename)).unwrap();
            {
                let msg = FileTransferAcceptMessage {};
                sender
                    .send_reliable(Messages::FileTransferAccept(msg))
                    .await
                    .unwrap();
            }
            let motd = download_loop(&handler, &mut file, &mut receiver)
                .await
                .unwrap();
            println!("MOTD: {}", motd);
            break;
        }
    }
}

pub async fn download_loop(
    handler: &NetworkHandler,
    file: &mut File,
    receiver: &mut Receiver<Messages>,
) -> Option<String> {
    loop {
        let msg = receiver.recv().await.unwrap();
        match msg {
            Messages::PartBegin(msg) => {
                let mut part_data = vec![0u8; msg.part_size as usize];
                part_loop(handler, file, &mut part_data, &msg, receiver)
                    .await
                    .unwrap();
            }
            Messages::Goodbye(msg) => {
                return Some(msg.motd);
            }
            _ => continue,
        }
    }
    Some("ERROR".to_string())
}

pub async fn part_loop(
    handler: &NetworkHandler,
    file: &mut File,
    part_data: &mut Vec<u8>,
    part_info: &PartBeginMessage,
    receiver: &mut Receiver<Messages>,
) -> Option<()> {
    let mut sender = handler.get_sender();

    let mut received_files = Bitfield::new();

    while let Ok(msg) = receiver.recv().await {
        match msg {
            Messages::Chunk(msg) => {
                if msg.part_number != part_info.part_number {
                    continue;
                }
                let range = (msg.index * part_info.chunk_size) as usize
                    ..(msg.index * part_info.chunk_size) as usize + msg.data.len();
                part_data.splice(range, msg.data);
                received_files.set(msg.index as usize, true);
            }
            Messages::PartEnd(_) => {
                let successful = received_files
                    .iter()
                    .take(part_info.chunk_count as usize)
                    .all(|a| a);
                if successful {
                    file.write_all(part_data).unwrap();
                    let msg = TransferSuccessfulMessage {};
                    sender
                        .send_reliable(Messages::TransferSuccessful(msg))
                        .await
                        .unwrap();
                    break;
                } else {
                    println!(
                        "{} Missing chunks... Fixing.",
                        received_files
                            .iter()
                            .take(part_info.chunk_count as usize)
                            .filter(|a| !a)
                            .count()
                    );
                    let msg = TransferIncompleteMessage {
                        bitfield: received_files.clone(),
                    };
                    sender
                        .send_reliable(Messages::TransferIncomplete(msg))
                        .await
                        .unwrap();
                }
            }
            _ => continue,
        }
    }
    Some(())
}
