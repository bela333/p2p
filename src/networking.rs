use std::{sync::{Mutex, atomic::{Ordering, AtomicU32}, Arc}, net::SocketAddr, time::Duration};
use crate::message::{PingMessage, Messages, ReliableMessage, ReliableAckMessage};
use tokio::sync::broadcast;
use tokio::net::UdpSocket;
use crate::{bitfield::Bitfield, error::Error};
use tokio_util::codec::Decoder;
use bytes::BytesMut;

pub struct NetworkHandler{
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    sender: broadcast::Sender<Messages>,
    receiver_in: broadcast::Sender<Messages>,
    index: Arc<AtomicU32>,
    received_messages: Arc<Mutex<Bitfield>>
}

pub struct Sender{
    sender: tokio::sync::broadcast::Sender<Messages>,
    receiver: tokio::sync::broadcast::Receiver<Messages>,
    index: Arc<AtomicU32>
}

impl Sender{
    pub fn send(&self, message: Messages) -> Result<(), Error>{
        self.sender.send(message).map_err(|_| Error::new("Packet send error"))?;
        Ok(())
    }

    pub async fn send_reliable(&mut self, message: Messages) -> Result<(), Error>{
        let index = self.index.fetch_add(1, Ordering::Relaxed);
        let mut ping_interval = tokio::time::interval(Duration::from_secs(3));
        loop {
            tokio::select!{
                msg = self.receiver.recv() => {
                    if let Ok(msg) = msg{
                        if let Messages::ReliableAck(msg) = msg{
                            if msg.packet_index == index{
                                break;
                            }
                        }
                    }
                }
                _ = ping_interval.tick() => {
                    let message = Box::new(message.clone());
                    self.send(Messages::Reliable(ReliableMessage{
                        packet_index: index,
                        message,
                    }))?;
                }
            }
        }
        Ok(())

    }
}

impl NetworkHandler{
    pub fn new(local_addr: SocketAddr, remote_addr: SocketAddr) -> Self{
        let (sender, _) = broadcast::channel::<Messages>(10);
        let (receiver_in, _) = broadcast::channel::<Messages>(10);
        NetworkHandler{
            local_addr, remote_addr, sender, receiver_in, index: Arc::new(AtomicU32::new(0)), received_messages: Arc::new(Mutex::new(Bitfield::new()))
        }
    }

    pub fn get_sender(&self) -> Sender{
        Sender{
            sender: self.sender.clone(),
            index: Arc::clone(&self.index),
            receiver: self.subscribe()
        }
    }



    pub async fn receive(&self) -> Result<Messages, Error>{
        self.subscribe().recv().await.map_err(|_| Error::new("Packet receive error"))
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Messages>{
        self.receiver_in.subscribe()
    }

    pub async fn begin(&self) -> Result<(), Error>{
        let client = UdpSocket::bind(self.local_addr).await?;
        client.connect(self.remote_addr).await?;
        let (mut udp_receiver, mut udp_sender) = client.split();
        let mut codec = Messages::Ping(PingMessage{});
        //Send messages to peer
        let mut sender_out = self.sender.subscribe();
        tokio::spawn(async move {
            while let Ok(msg) = sender_out.recv().await{
                let bytes_vec = msg.get_bytes();
                let bytes = bytes_vec.as_slice();
                udp_sender.send(bytes).await.unwrap();
            }
        });
        //Receive messages from peer
        let receiver_in = self.receiver_in.clone();
        tokio::spawn(async move{
            let mut buf = [0; 2048];
            
            while let Ok(size) = udp_receiver.recv(&mut buf).await {
                if size < 4{
                    continue;
                }
                let mut bytes = BytesMut::new();
                bytes.extend(buf.iter().take(size));
                let data = codec.decode(&mut bytes).ok().unwrap().unwrap(); //TODO: replace unwrap
                receiver_in.send(data).map_err(|_| Error::new("Broadcast send error")).unwrap();
            }
        });
        //Handle reliable messages
        let mut receiver = self.subscribe();
        let received_messages = Arc::clone(&self.received_messages);
        let receiver_in = self.receiver_in.clone();
        let sender = self.get_sender();
        tokio::spawn(async move{
            while let Ok(msg) = receiver.recv().await {
                if let Messages::Reliable(msg) = msg{
                    let mut received_messages = received_messages.lock().unwrap();
                    if !received_messages.get(msg.packet_index as usize){
                        received_messages.set(msg.packet_index as usize, true);
                        receiver_in.send(*msg.message).map_err(|_| Error::new("Broadcast send error")).unwrap();
                    }
                    
                    sender.send(Messages::ReliableAck(ReliableAckMessage{
                        packet_index: msg.packet_index
                    })).unwrap();
                }
            }
        });
        let ping_sender = self.get_sender();
        //Send pings
        tokio::spawn(async move {
            let mut ping_interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                ping_interval.tick().await;
                ping_sender.send(Messages::Ping(PingMessage{})).unwrap();
            }
        });
        Ok(())
    }

    

    pub async fn wait_for_connection(&self){
        let mut receiver = self.subscribe();
        while let Ok(message) =  receiver.recv().await {
            if let Messages::Ping(_) = message{
                break;
            }
        }
    }
}