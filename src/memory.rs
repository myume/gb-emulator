pub struct MMU {}

impl MMU {
    pub fn new() -> Self {
        MMU {}
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        todo!()
    }
    pub fn write_byte(&self, address: u16, value: u8) {}

    pub fn read_word(&self, address: u16) -> u16 {
        todo!()
    }
    pub fn write_word(&self, address: u16, value: u16) {}
}
