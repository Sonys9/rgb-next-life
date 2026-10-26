use core::slice::from_raw_parts;

use esp_hal::peripherals::FLASH;
use esp_storage::FlashStorage;

use crate::{bytes::{align, utf8}, errors::Error};

const MEM_OFFSET: u32 = 0x3F0000;
const SECTOR: u32 = 4096;

pub struct Storage<'a> {
    flash: FlashStorage<'a>,
    read_buffer: &'a mut [u8],
    write_buffer: &'a mut [u8],
}

impl<'a> Storage<'a> {
    pub fn new(flash: FLASH<'a>, read_buffer: &'a mut [u8], write_buffer: &'a mut [u8]) -> Self {
        Self { flash: FlashStorage::new(flash), read_buffer, write_buffer }
    }

    pub fn erase(&mut self) -> Result<(), Error> {
        self.flash.erase(MEM_OFFSET, MEM_OFFSET + SECTOR).map_err(|_| Error::StorageErase)
    }

    pub fn write_slice(&mut self, offset: u32, content: &[u8]) -> Result<usize, Error> {
        let align_to = (content.len() + 3) & !3;
        align(content, align_to, self.write_buffer)?;
        self.flash.write(MEM_OFFSET + offset, &content.len().to_be_bytes()).map_err(|_| Error::StorageWrite)?;
        self.flash.write(MEM_OFFSET + 4 + offset, &self.write_buffer[..align_to]).map_err(|_| Error::StorageWrite)?;
        Ok(4 + align_to)
    }

    pub fn update<'b>(&mut self, auth: Auth<'b>) -> Result<(), Error> {
        if auth.ssid.len() + auth.password.len() > SECTOR as usize {
            return Err(Error::TooBig);
        };
        self.erase()?;
        let len = self.write_slice(0, auth.ssid.as_bytes())?;
        self.write_slice(len as u32, auth.password.as_bytes())?;
        Ok(())
    }

    pub fn read_slice(&self, offset: usize) -> Result<(&'a [u8], usize), Error> {
        let len = usize::from_be_bytes(self.read_buffer[offset..offset + 4].try_into().unwrap());
        let aligned_len = (len + 3) & !3;
        let result = unsafe { from_raw_parts(self.read_buffer.as_ptr().add(offset + 4), len)};
        Ok((result, aligned_len - len))
    }

    pub fn read(&mut self) -> Result<Auth<'a>, Error> {
        self.flash.read(MEM_OFFSET, self.read_buffer).map_err(|_| Error::StorageRead)?;
        let (ssid, skip) = self.read_slice(0)?;
        let password_pos = ssid.len() + 4 + skip;
        let (password, _) = self.read_slice(password_pos)?;
        Ok(Auth { ssid: utf8(ssid)?, password: utf8(password)? })
    }
}

#[derive(Debug)]
pub struct Auth<'b> {
    pub ssid: &'b str,
    pub password: &'b str
}
