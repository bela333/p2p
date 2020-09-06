use std::net::SocketAddr;
use crate::message::{PingMessage, Messages};
use tokio::sync::broadcast;
use tokio::net::UdpSocket;
use crate::error::Error;
use tokio_util::codec::Decoder;
use bytes::BytesMut;

pub struct NetworkHandler{
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    sender: broadcast::Sender<Messages>,
    receiver_in: broadcast::Sender<Messages>
}

impl NetworkHandler{
    pub fn new(local_addr: SocketAddr, remote_addr: SocketAddr) -> Self{
        let (sender, _) = broadcast::channel::<Messages>(1);
        let (receiver_in, _) = broadcast::channel::<Messages>(1);
        NetworkHandler{
            local_addr, remote_addr, sender, receiver_in
        }
    }

    pub fn send(&self, message: Messages) -> Result<(), Error>{
        self.sender.send(message).map_err(|_| Error::new("Packet send error"))?;
        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Messages, Error>{
        self.receiver_in.subscribe().recv().await.map_err(|_| Error::new("Packet receive error"))
    }

    pub async fn begin(&mut self) -> Result<(), Error>{
        let client = UdpSocket::bind(self.local_addr).await?;
        client.connect(self.remote_addr).await?;
        let (mut udp_receiver, mut udp_sender) = client.split();
        let mut codec = Messages::Ping(PingMessage{});
        //Send messages to peer
        let mut sender_out = self.sender.subscribe();
        tokio::spawn(async move {
            while let Ok(msg) = sender_out.recv().await{
                udp_sender.send(msg.get_bytes().as_slice()).await.unwrap();
            }
        });

        let receiver_in = self.receiver_in.clone();
        tokio::spawn(async move{
            let mut buf = [0; 2048];
            
            while let Ok(_) = udp_receiver.recv(&mut buf).await {
                let mut bytes = BytesMut::new();
                bytes.extend(buf.iter());
                let data = codec.decode(&mut bytes).ok().unwrap().unwrap(); //TODO: replace unwrap
                receiver_in.send(data);
            }
        });
        Ok(())
    }
}