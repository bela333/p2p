use std::{path::Path, fs::File, io::Read, slice, io::BufRead, io::BufReader, iter};
//TODO: More errors, instead of Option

use crate::{networking::NetworkHandler, message::Messages, message::FileTransferRequestMessage, message::PartBeginMessage, message::ChunkMessage, message::PartEndMessage};

pub async fn begin(handler: NetworkHandler, path: &Path){
    let file = File::open(path).unwrap();
    let mut partno = 0u32;
    
    handler.wait_for_connection().await;
    println!("Connected!");
    let mut receiver = handler.subscribe();
    let mut sender = handler.get_sender();

    //Send Transfer Request
    {
        let msg = FileTransferRequestMessage{
            filename: path.file_name().unwrap().to_str().unwrap().to_string(),
            filesize: file.metadata().unwrap().len()
        };
        sender.send_reliable(Messages::FileTransferRequest(msg)).await.unwrap();
    }
    

    //Waiting for the request to be accepted
    while let Ok(msg) = receiver.recv().await {
        if let Messages::FileTransferAccept(_) = msg{
            break;
        }
    }

    let mut reader = BufReader::new(file);

    while let Some(_) = send_part(&handler, &mut reader, 1024, 8*1024, partno).await {partno += 1};
    println!("Finished sending the file!");
}

pub async fn send_part<T: BufRead>(handler: &NetworkHandler, file: &mut T, chunk_size: u32, chunk_count: u32, partno: u32) -> Option<()> {
    let mut sender = handler.get_sender();
    let mut receiver = handler.subscribe();

    let mut chunk_count = chunk_count;
    let chunks = split(file, chunk_size as usize, &mut chunk_count)?;
    if chunk_count <= 0{
        return None;
    }

    println!("Sending part {}", partno);
    {
        let msg = Messages::PartBegin(PartBeginMessage{
            chunk_count,
            chunk_size,
            part_number: partno
        });
        sender.send_reliable(msg).await.unwrap();
    }
    
    send_chunks(handler, &chunks, &mut iter::repeat(false), partno);

    {
        let msg = Messages::PartEnd(PartEndMessage{});
        sender.send_reliable(msg).await.unwrap();
    }
    
    println!("Waiting for response");
    while let Ok(msg) = receiver.recv().await {
        match msg {
            Messages::TransferIncomplete(msg) =>{
                send_chunks(handler, &chunks, &mut msg.bitfield.iter(), partno);
                {
                    let msg = Messages::PartEnd(PartEndMessage{});
                    sender.send_reliable(msg).await.unwrap();
                }
            },
            Messages::TransferSuccessful(_) => break,
            _ => continue
        }
    }

    Some(())
}

fn send_chunks<T: Iterator<Item = bool>>(handler: &NetworkHandler, chunks: &Vec<Vec<u8>>, mask: &mut T, partno: u32)-> Option<()>{
    let sender = handler.get_sender();
    for (i, chunk) in (*chunks).iter().enumerate(){
        if !(mask.next()?){
            //This chunk was NOT skipped
            let msg = ChunkMessage{
                data: chunk.to_owned(),
                index: i as u32,
                part_number:partno
            };
            sender.send(Messages::Chunk(msg)).unwrap();
        }
    }
    Some(())
}

fn split<T: BufRead>(file: &mut T, chunk_size: usize, chunk_count:&mut u32) -> Option<Vec<Vec<u8>>>{
    let mut v = Vec::new();
    for i in 0..(*chunk_count){
        let mut buf = vec![0;chunk_size];
        let slice = buf.as_mut_slice();
        let c = file.read(slice).unwrap();
        if c <= 0{
            break;
        }
        v.push(buf);
    }
    *chunk_count = v.len() as u32;
    Some(v)
}