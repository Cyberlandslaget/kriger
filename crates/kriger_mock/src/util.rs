pub fn encode(buf: &mut [u8]) {
    for ch in buf {
        let x = *ch % 36;
        if x < 26 {
            *ch = b'A' + x;
        } else {
            *ch = b'0' + x;
        }
    }
}

pub fn decode(buf: &mut [u8]) -> Option<()> {
    for ch in buf {
        if ch.is_ascii_alphanumeric() {
            if ch.is_ascii_uppercase() {
                *ch = *ch - b'A';
            } else if ch.is_ascii_digit() {
                *ch = *ch - b'0' + 26;
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    Some(())
}

pub fn encode_u32(buf: &mut [u8], mut n: u32) {
    for i in 0..7 {
        buf[i] = (n % 36) as u8;
        n /= 36;
    }
    encode(buf);
}

pub fn decode_u32(buf: &mut [u8]) -> Option<u32> {
    decode(buf)?;
    let mut n = 0;
    for i in 0..7 {
        n *= 36;
        n += buf[6 - i] as u32;
    }
    Some(n)
}
