pub fn decode_hex(s: &str) -> (u8, u8, u8) {
    let mut result: (u8, u8, u8) = (0, 0, 0);
    result.0 = u8::from_str_radix(&s[1..3], 16).unwrap_or(0);
    result.1 = u8::from_str_radix(&s[3..5], 16).unwrap_or(0);
    result.2 = u8::from_str_radix(&s[5..7], 16).unwrap_or(0);
    result
}
