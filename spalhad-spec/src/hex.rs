use core::fmt;

#[must_use]
pub fn parse(input: &str, buf: &mut [u8]) -> bool {
    let mut chars = input.chars();
    let mut bytes = buf.iter_mut();
    loop {
        let Some(byte) = bytes.next() else {
            return chars.next().is_none();
        };
        let Some(high) = chars.next() else { return false };
        let Some(low) = chars.next() else { return false };
        let Some(high) = high.to_digit(16) else { return false };
        let Some(low) = low.to_digit(16) else { return false };
        *byte = (low | (high << 4)) as u8;
    }
}

pub fn render(buf: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
    for byte in buf {
        write!(f, "{:02x}", byte)?;
    }
    Ok(())
}
