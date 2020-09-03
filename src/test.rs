#[cfg(test)]
mod tests{
    use crate::obfuscator::AddressInfo;
    use std::net::SocketAddrV4;

    #[test]
    fn bytes_to_string(){
        let addr = SocketAddrV4::new("192.168.0.111".parse().unwrap(), 6543);
        println!("Address: {}", addr);
        let info = AddressInfo::new(addr);
        let code = info.to_string();
        println!("Code: {}", code);
        let info: AddressInfo = code.parse().unwrap();
        assert_eq!(info.address, addr);

    }
}