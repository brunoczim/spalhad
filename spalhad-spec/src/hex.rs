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

pub fn divide_le<const N: usize>(
    dividend: &[u8; N],
    divisor: &[u8; N],
    quotient: &mut [u8; N],
    remainder: &mut [u8; N],
) {
    for byte in &mut *remainder {
        *byte = 0;
    }
    for byte in &mut *quotient {
        *byte = 0;
    }
    for i in (0 .. N * 8).rev() {
        let mut carry = 0;
        for byte in &mut *remainder {
            let new_byte = (*byte << 1) | carry;
            carry = *byte >> 7;
            *byte = new_byte;
        }
        let mut carry = 0;
        for byte in &mut *quotient {
            let new_byte = (*byte << 1) | carry;
            carry = *byte >> 7;
            *byte = new_byte;
        }
        remainder[0] |= (dividend[i / 8] >> (i % 8)) & 1;
        if (*remainder).into_iter().rev().ge((*divisor).into_iter().rev()) {
            quotient[0] |= 1;
            let mut borrow = 0;
            for (dest, src) in remainder.iter_mut().zip((*divisor).into_iter())
            {
                let (byte, borrow_a) = dest.overflowing_sub(src);
                let (byte, borrow_b) = byte.overflowing_sub(borrow);
                *dest = byte;
                borrow = u8::from(borrow_a | borrow_b);
            }
        }
    }
}
