use std::iter;

#[derive(Clone)]
pub struct Bitfield{
    bytes: Vec<u8>
}

impl Bitfield {
    pub fn new() -> Self{
        Self{
            bytes: Vec::new()
        }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self{
        Self{
            bytes
        }
    }

    pub fn get_bytes(&self) -> &Vec<u8>{
        &self.bytes
    }

    pub fn set(&mut self, idx: usize, value: bool) {
        self.ensure(idx);
        let array_index = idx/8;
        let bit_index = idx%8;
        let bit_mask = 1 << bit_index;
        let mut val = self.bytes[array_index]& (!bit_mask);
        if value{
            val |= bit_mask;
        }
        self.bytes[array_index] =  val;
    }

    pub fn get(&self, idx: usize) -> bool{
        let required_size = idx/8+1;
        if self.bytes.len() < required_size{
            return false;
        }
        let array_index = idx/8;
        let bit_index = idx%8;
        let bit_mask = 1 << bit_index;
        (self.bytes[array_index] & bit_mask) != 0
    }

    fn ensure(&mut self, idx: usize){
        let required_size = idx/8+1;
        if self.bytes.len() < required_size{
            self.bytes.extend(iter::repeat(0).take(required_size - self.bytes.len()))
        }
    }
}