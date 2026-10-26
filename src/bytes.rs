use crate::errors::Error;

pub fn sum<'a>(
    a: &'a [u8],
    b: &'a [u8],
    result_buf: &'a mut [u8],
) -> Result<(), Error> {
    if result_buf.len() < a.len() + b.len() {
        return Err(Error::Buffer);
    };

    for (i, byte) in a.iter().enumerate() {
        result_buf[i] = *byte;
    }

    for (i, byte) in a.iter().enumerate() {
        result_buf[i + a.len()] = *byte;
    }
    
    Ok(())
}