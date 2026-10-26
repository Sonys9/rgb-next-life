use core::str::from_utf8;

use crate::errors::Error;

pub fn align<'a>(slice: &'a [u8], to: usize, result_buf: &'a mut [u8]) -> Result<(), Error> {
    if result_buf.len() < to {
        return Err(Error::Buffer);
    };

    for (i, byte) in slice.iter().enumerate() {
        result_buf[i] = *byte;
    }

    for i in slice.len()..to {
        result_buf[i] = b'0';
    }

    Ok(())
}

pub fn utf8<'a>(slice: &'a [u8]) -> Result<&'a str, Error> {
    from_utf8(slice).map_err(|_| Error::Parse)
}
