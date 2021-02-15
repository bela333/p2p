use std::{
    fs::File, io::BufRead, io::BufReader, io::Read, iter, path::Path, slice, thread::sleep,
    time::Duration,
};
//TODO: More errors, instead of Option

use crate::{
    message::ChunkMessage,
    message::FileTransferRequestMessage,
    message::Messages,
    message::PartBeginMessage,
    message::{GoodbyeMessage, PartEndMessage},
    networking::NetworkHandler,
};

pub async fn begin(handler: NetworkHandler, path: &Path) {
    let file = File::open(path).unwrap();
    let mut partno = 0u32;

    handler.wait_for_connection().await;
    println!("Connected!");
    let mut receiver = handler.subscribe();
    let mut sender = handler.get_sender();
    println!("Sending file request!");
    //Send Transfer Request
    {
        let msg = FileTransferRequestMessage {
            filename: path.file_name().unwrap().to_str().unwrap().to_string(),
            filesize: file.metadata().unwrap().len(),
        };
        sender
            .send_reliable(Messages::FileTransferRequest(msg))
            .await
            .unwrap();
    }

    println!("Waiting for response...!");
    //Waiting for the request to be accepted
    while let Ok(msg) = receiver.recv().await {
        if let Messages::FileTransferAccept(_) = msg {
            break;
        }
    }
    let mut reader = BufReader::new(file);
    println!("Sending parts...!");
    while let Some(_) = send_part(&handler, &mut reader, 1024, 512, partno).await {
        //TODO: change hardwritten values
        partno += 1
    }
    println!("Finished sending the file!");
    {
        let msg = GoodbyeMessage {
            motd: "Thank you for using our service!".to_string(),
        };
        sender.send_reliable(Messages::Goodbye(msg)).await.unwrap();
    }
}

pub async fn send_part<T: BufRead>(
    handler: &NetworkHandler,
    file: &mut T,
    chunk_size: u32,
    chunk_count: u32,
    partno: u32,
) -> Option<()> {
    let mut sender = handler.get_sender();
    let mut receiver = handler.subscribe();

    let mut chunk_count = chunk_count;
    let chunks = split(file, chunk_size as usize, &mut chunk_count)?;
    if chunk_count <= 0 {
        return None;
    }

    {
        let msg = Messages::PartBegin(PartBeginMessage {
            part_size: chunks.iter().map(|chunk| chunk.len()).sum::<usize>() as u32,
            chunk_size,
            part_number: partno,
            chunk_count,
        });
        sender.send_reliable(msg).await.unwrap();
    }

    send_chunks(handler, &chunks, &mut iter::repeat(false), partno);

    {
        let msg = Messages::PartEnd(PartEndMessage {});
        sender.send_reliable(msg).await.unwrap();
    }

    while let Ok(msg) = receiver.recv().await {
        match msg {
            Messages::TransferIncomplete(msg) => {
                send_chunks(handler, &chunks, &mut msg.bitfield.iter(), partno);
                {
                    let msg = Messages::PartEnd(PartEndMessage {});
                    sender.send_reliable(msg).await.unwrap();
                }
            }
            Messages::TransferSuccessful(_) => break,
            _ => continue,
        }
    }

    Some(())
}

fn send_chunks<T: Iterator<Item = bool>>(
    handler: &NetworkHandler,
    chunks: &Vec<Vec<u8>>,
    mask: &mut T,
    partno: u32,
) -> Option<()> {
    let sender = handler.get_sender();
    for (i, chunk) in (*chunks).iter().enumerate() {
        if !(mask.next()?) {
            //This chunk was NOT skipped
            let msg = ChunkMessage {
                data: chunk.to_owned(),
                index: i as u32,
                part_number: partno,
            };
            sender.send(Messages::Chunk(msg)).unwrap();
        }
    }
    Some(())
}

fn split<T: BufRead>(
    file: &mut T,
    chunk_size: usize,
    chunk_count: &mut u32,
) -> Option<Vec<Vec<u8>>> {
    let mut v = Vec::new();
    for i in 0..(*chunk_count) {
        let mut buf = vec![0; chunk_size];
        let slice = buf.as_mut_slice();
        let c = file.read(slice).unwrap();
        if c <= 0 {
            break;
        }
        buf.truncate(c);
        v.push(buf);
    }
    *chunk_count = v.len() as u32;
    Some(v)
}
