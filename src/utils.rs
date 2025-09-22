pub fn compose_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | low as u16
}

pub fn is_set(byte: u8, index: u8) -> bool {
    byte & 1 << index > 0
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
}
