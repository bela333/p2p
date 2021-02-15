#[cfg(test)]
mod tests {
    use crate::{bitfield::Bitfield, obfuscator::AddressInfo};
    use std::net::SocketAddrV4;

    #[test]
    fn bytes_to_string() {
        let addr = SocketAddrV4::new("192.168.0.111".parse().unwrap(), 6543);
        println!("Address: {}", addr);
        let info = AddressInfo::new(addr);
        let code = info.to_string();
        println!("Code: {}", code);
        let info: AddressInfo = code.parse().unwrap();
        assert_eq!(info.address, addr);
    }

    #[test]
    fn bitfield() {
        let mut bitfield = Bitfield::new();
        bitfield.set(1, true);
        bitfield.set(0, true);
        bitfield.set(3, true);
        assert_eq!(bitfield.get_bytes()[0], 0b00001011);
        bitfield.set(2, false);
        assert_eq!(bitfield.get_bytes()[0], 0b00001011);
        bitfield.set(3, false);
        assert_eq!(bitfield.get_bytes()[0], 0b00000011);

        for (idx, &value) in [true, true, false, false, false, false, false, false]
            .iter()
            .enumerate()
        {
            assert_eq!(bitfield.get(idx), value);
        }

        let iter = bitfield.iter();
        assert!(iter.take(2).all(|a| a == true));
        let iter = bitfield.iter();
        assert!(iter.skip(2).take(6).all(|a| a == false));
        let iter = bitfield.iter();
        assert!(iter.skip(2).take(100).all(|a| a == false));

        bitfield.set(145, true);
        assert_eq!(bitfield.get(145), true);
        assert_eq!(bitfield.get_bytes()[145 / 8], 0b00000010);
    }
}
