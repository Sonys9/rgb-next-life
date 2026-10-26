use core::slice::from_raw_parts;

use embassy_time::Timer;
use esp_hal::peripherals::FLASH;
use esp_println::println;
use esp_radio::wifi::{self, AuthenticationMethodConfig, Password};
use esp_storage::FlashStorage;
use serde::{Deserialize, Deserializer, de::{IntoDeserializer, value::U8Deserializer}};

use crate::{
    bytes::{align, utf8}, errors::Error, network::{Auth, AuthMethod},
};

const MEM_OFFSET: u32 = 0x3F0000;
const SECTOR: u32 = 4096;

pub struct Storage<'a> {
    flash: FlashStorage<'a>,
    read_buffer: &'a mut [u8],
    write_buffer: &'a mut [u8],
}

impl<'a> Storage<'a> {
    pub fn new(flash: FLASH<'a>, read_buffer: &'a mut [u8], write_buffer: &'a mut [u8]) -> Self {
        Self {
            flash: FlashStorage::new(flash),
            read_buffer,
            write_buffer,
        }
    }

    pub fn erase(&mut self) -> Result<(), Error> {
        self.flash
            .erase(MEM_OFFSET, MEM_OFFSET + SECTOR)
            .map_err(|_| Error::StorageErase)
    }

    pub fn write_slice(&mut self, offset: u32, content: &[u8]) -> Result<usize, Error> {
        let align_to = (content.len() + 3) & !3;
        align(content, align_to, self.write_buffer)?;
        self.flash
            .write(MEM_OFFSET + offset, &content.len().to_be_bytes())
            .map_err(|_| Error::StorageWrite)?;
        self.flash
            .write(MEM_OFFSET + 4 + offset, &self.write_buffer[..align_to])
            .map_err(|_| Error::StorageWrite)?;
        Ok(4 + align_to)
    }

    pub fn update<'b>(&mut self, auth: &Auth) -> Result<(), Error> {
        if auth.ssid.len() + auth.password.len() > SECTOR as usize {
            return Err(Error::TooBig);
        };
        self.erase()?;
        let ssid_len = self.write_slice(0, auth.ssid.as_bytes())?;
        let password_len = self.write_slice(ssid_len as u32, auth.password.as_bytes())?;
        self.write_buffer[0] = u8::try_from(&auth.auth_method).map_err(|_| Error::Parse)?;
        self.flash
            .write(MEM_OFFSET + ssid_len as u32 + password_len as u32, &self.write_buffer[..4])
            .map_err(|_| Error::StorageWrite)?;
        Ok(())
    }

    pub fn read_slice(&'a self, offset: usize) -> Result<(&'a [u8], usize), Error> {
        let len = usize::from_be_bytes(self.read_buffer[offset..offset + 4].try_into().unwrap());
        if len == usize::MAX {
            return Ok((b"", 0))
        };
        let aligned_len = (len + 3) & !3;
        let result = &self.read_buffer[offset + 4..offset + 4 + len];
        Ok((result, aligned_len))
    }

    pub async fn read(&mut self) -> Result<Auth, Error> {
        self.flash
            .read(MEM_OFFSET, self.read_buffer)
            .map_err(|_| Error::StorageRead)?;
        println!("{:?}",&self.read_buffer[..64]);
        let (ssid, skip) = self.read_slice(0)?;
        let password_pos = 4 + skip;
        let (password, skip) = self.read_slice(password_pos)?;
        let auth_method = AuthMethod::try_from(self.read_buffer[password_pos + skip + 4]).map_err(|_| Error::Parse)?;
        
        Ok(Auth {
            ssid: utf8(ssid)?.try_into().map_err(|_| Error::Parse)?,
            password: utf8(password)?.try_into().map_err(|_| Error::Parse)?,
            auth_method,
        })
    }
}

