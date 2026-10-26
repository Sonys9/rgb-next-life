use embassy_embedded_hal::adapter::BlockingAsync;
use esp_hal::peripherals::FLASH;
use esp_storage::FlashStorage;
use embedded_storage_async::nor_flash::{NorFlash, ReadNorFlash};

use crate::errors::Error;

const MEM_OFFSET: u32 = 0x3F0000;
const SECTOR: u32 = 4096;

pub struct Storage<'a> {
    flash: BlockingAsync<FlashStorage<'a>>,
    read_buffer: &'a mut [u8],
    write_buffer: &'a mut [u8],
}

impl<'a> Storage<'a> {
    pub fn new(flash: FLASH<'a>, read_buffer: &'a mut [u8], write_buffer: &'a mut [u8]) -> Self {
        Self { flash: BlockingAsync::new(FlashStorage::new(flash)), read_buffer, write_buffer }
    }

    pub async fn erase(&mut self) -> Result<(), Error> {
        self.flash.erase(MEM_OFFSET, MEM_OFFSET + SECTOR).await.map_err(|_| Error::Erase)
    }

    pub async fn write_slice(&mut self, offset: u32, content: &[u8]) -> Result<usize, Error> {
        self.flash.write(MEM_OFFSET + offset, &content.len().to_be_bytes()).await.map_err(|_| Error::Write)?;
        let skip_bytes = ((content.len() + 3) & !3) - content.len();
        self.flash.write(MEM_OFFSET + 4 + offset + skip_bytes as u32, content).await.map_err(|_| Error::Write)?;
        Ok(4 + content.len())
    }

    pub async fn update<'b>(&mut self, auth: Auth<'b>) -> Result<(), Error> {
        if auth.ssid.len() + auth.password.len() > SECTOR as usize {
            return Err(Error::TooBig);
        };
        self.erase().await?;
        let len = self.write_slice(0, auth.ssid).await?;
        self.write_slice(len as u32, auth.password).await?;
        Ok(())
    }

    pub async fn read_slice(&mut self, offset: u32) -> Result<&[u8], Error> {
        let mut buffer = [0u8; 4];
        self.flash.read(offset, &mut buffer).await.unwrap();
        let len = usize::from_be_bytes(buffer);
        self.flash.read(offset, &mut self.read_buffer[..(len + 3) & !3]).await.unwrap();
        info!("{:?}", self.read_buffer);
        Ok(&[])
    }

    pub async fn read(&mut self, ssid: &[u8], password: &[u8]) -> Result<Auth<'a>, Error> {
        if ssid.len() + password.len() > SECTOR as usize {
            return Err(Error::TooBig);
        };
        self.erase().await?;
        let len = self.write_slice(MEM_OFFSET, ssid).await?;
        self.write_slice(MEM_OFFSET + len as u32, password).await?;
        Ok(Auth { ssid: b"", password: b"" })
    }
}

pub struct Auth<'b> {
    pub ssid: &'b [u8],
    pub password: &'b [u8]
}
