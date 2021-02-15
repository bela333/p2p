use crate::error::Error;
use std::{
    fmt::Display,
    io::Write,
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};
pub struct AddressInfo {
    pub address: SocketAddrV4,
}

impl AddressInfo {
    pub fn new(socket: SocketAddrV4) -> Self {
        Self { address: socket }
    }

    pub fn new_from_address(addr: &str, port: u16) -> Self {
        Self::new(SocketAddrV4::new(addr.parse().unwrap(), port))
    }

    fn from_bytes(b: Vec<u8>) -> Result<Self, Error> {
        if b.len() < 6 {
            return Err(Error::new("Invalid data"));
        }
        let addr = Ipv4Addr::new(b[0], b[1], b[2], b[3]);
        let port = u16::from_le_bytes([b[4], b[5]]);
        Ok(AddressInfo {
            address: SocketAddrV4::new(addr, port),
        })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        buf.write(&self.address.ip().octets()).unwrap();
        buf.write(&self.address.port().to_le_bytes()).unwrap();
        buf
    }
}

impl FromStr for AddressInfo {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let byte_data = bs58::decode(s).into_vec()?;
        if byte_data.len() != 6 {
            return Err(Error::new("Invalid size"));
        }
        AddressInfo::from_bytes(byte_data)
    }
}

impl Display for AddressInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&bs58::encode(self.to_bytes()).into_string())
    }
}
