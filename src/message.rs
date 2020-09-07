use tokio_util::codec::Decoder;
use crate::error::Error;
use bytes::BytesMut;
use std::convert::TryInto;

#[derive(Clone)]
pub enum Messages{
    Reliable(ReliableMessage),
    ReliableAck(ReliableAckMessage),
    Ping(PingMessage),
    Chat(ChatMessage)
}

impl Decoder for Messages {
    type Item = Messages;

    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let message_id = u32::from_le_bytes(src[0..4].try_into()?);
        let rest = src[4..].to_vec();
        let message = match message_id {
            0 => Messages::Reliable(*(ReliableMessage::from_bytes(rest).ok_or(Error::new("Invalid data"))?)),
            1 => Messages::ReliableAck(*(ReliableAckMessage::from_bytes(rest).ok_or(Error::new("Invalid data"))?)),
            2 => Messages::Ping(*(PingMessage::from_bytes(rest).ok_or(Error::new("Invalid data"))?)),
            3 => Messages::Chat(*(ChatMessage::from_bytes(rest).ok_or(Error::new("Invalid data"))?)),
            _ => return Err(Error::new("Invalid packet ID"))
        };
        Ok(Some(message))
    }
}

impl Messages{
    pub fn get_bytes(&self) -> Vec<u8>{
        match self {
            Messages::Reliable(a) => {a.get_bytes()}
            Messages::ReliableAck(a) => {a.get_bytes()}
            Messages::Ping(a) => {a.get_bytes()}
            Messages::Chat(a) => {a.get_bytes()}
        }
    }
}

pub trait Message{
    const ID: u32;
    fn get_data(&self) -> Vec<u8>;
    fn get_bytes(&self) -> Vec<u8>{
        let mut buf: Vec<u8> = Vec::new();
        buf.extend(Self::ID.to_le_bytes().iter());
        buf.append(&mut self.get_data());
        buf
    }
    fn from_bytes(bytes: Vec<u8>) -> Option<Box<Self>>;
}

#[derive(Clone)]
pub struct ReliableMessage{
    pub packet_index: u32,
    pub message: Box<Messages>
}

impl Message for ReliableMessage{
    const ID: u32 = 0;

    fn get_data(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.extend(self.packet_index.to_le_bytes().iter());
        buf.append(&mut self.message.get_bytes());
        buf
    }

    fn from_bytes(bytes: Vec<u8>) -> Option<Box<Self>> {
        let index = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
        let mut buf = BytesMut::new();
        buf.extend(bytes[4..].iter());
        let message = Messages::Ping(PingMessage{}).decode(&mut buf).ok()??;
        Some(Box::new(Self{
            packet_index: index,
            message: Box::new(message)
        }))
    }
}

#[derive(Clone)]
pub struct ReliableAckMessage{
    pub packet_index: u32
}

impl Message for ReliableAckMessage{
    const ID: u32 = 1;

    fn get_data(&self) -> Vec<u8> {
        self.packet_index.to_le_bytes().to_vec()
    }

    fn from_bytes(bytes: Vec<u8>) -> Option<Box<Self>> {
        let index = u32::from_le_bytes(bytes[0..4].try_into().ok()?); 
        Some(Box::new(Self{
            packet_index: index
        }))
    }
}

#[derive(Clone)]
pub struct PingMessage{}
impl Message for PingMessage {
    const ID: u32 = 2;
    fn get_data(&self) -> Vec<u8> { Vec::new() }

    fn from_bytes(bytes: Vec<u8>) -> Option<Box<Self>> {
        Some(Box::new(Self{}))
    }
}

#[derive(Clone)]
pub struct ChatMessage{
    pub message: String
}

impl Message for ChatMessage {
    const ID: u32 = 3;

    fn get_data(&self) -> Vec<u8> {
        self.message.as_bytes().to_vec()
    }

    fn from_bytes(bytes: Vec<u8>) -> Option<Box<Self>> {
        Some(Box::new(Self{
            message: String::from_utf8(bytes).ok()?
        }))
    }
}