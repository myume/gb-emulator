pub fn compose_bytes(high: u8, low: u8) -> u16 {
    ((high as u16) << 8) | low as u16
}
