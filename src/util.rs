pub fn parse_hex_or_dec_u16(input: &str) -> std::result::Result<u16, String> {
    if let Some(stripped) = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
    {
        u16::from_str_radix(stripped, 16).map_err(|e| e.to_string())
    } else {
        input.parse::<u16>().map_err(|e| e.to_string())
    }
}

pub fn parse_hex_or_dec_u8(input: &str) -> std::result::Result<u8, String> {
    if let Some(stripped) = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
    {
        u8::from_str_radix(stripped, 16).map_err(|e| e.to_string())
    } else {
        input.parse::<u8>().map_err(|e| e.to_string())
    }
}

pub fn format_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn format_usage(usage_page: Option<u16>, usage: Option<u16>) -> String {
    match (usage_page, usage) {
        (Some(page), Some(usage)) => format!("0x{page:04x}:0x{usage:04x}"),
        (Some(page), None) => format!("0x{page:04x}:-"),
        (None, Some(usage)) => format!("-:0x{usage:04x}"),
        (None, None) => "-".to_string(),
    }
}
