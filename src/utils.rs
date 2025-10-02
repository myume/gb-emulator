pub fn compose_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | low as u16
}

pub fn is_set(byte: u8, index: u8) -> bool {
    byte & 1 << index > 0
}

pub fn set_bit(byte: u8, index: u8) -> u8 {
    byte | 1 << index
}

pub fn reset_bit(byte: u8, index: u8) -> u8 {
    byte & !(1 << index)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_set() {
        let byte = 0b01110011;
        assert!(is_set(byte, 0));
        assert!(is_set(byte, 1));
        assert!(!is_set(byte, 2));
        assert!(is_set(byte, 6));
        assert!(!is_set(byte, 7));
    }

    #[test]
    fn test_set_bit() {
        let byte = 0x00;
        assert_eq!(set_bit(byte, 0), 0x01);
        assert_eq!(set_bit(byte, 7), 0b10000000);
    }

    #[test]
    fn test_reset_bit() {
        let byte = 0xFF;
        assert_eq!(reset_bit(byte, 0), 0xFE);
        assert_eq!(reset_bit(byte, 7), 0x7F);
    }
}
