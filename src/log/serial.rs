use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::{Channel, Receiver, Sender}, mutex::{Mutex, MutexGuard, TryLockError}};
use embassy_time::{Duration, with_timeout};
use embedded_io_async::Write;
use esp_hal::{
    Async, peripherals::USB_DEVICE, usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagRx, UsbSerialJtagTx},
};
use crate::errors::Error;

pub const CHANNEL_BUFFER: usize = 32;
pub const BUFFER_SIZE: usize = 2048;

pub static SERIAL: Mutex<CriticalSectionRawMutex, Option<Serial>> = Mutex::new(None);
static CHANNEL: Channel<CriticalSectionRawMutex, Command, CHANNEL_BUFFER> = Channel::new();
static BUFFER: Mutex<CriticalSectionRawMutex, BufferData> = Mutex::new( BufferData { buffer: [0u8; BUFFER_SIZE], size: 0 } );

#[derive(Copy, Clone)]
struct BufferData {
    buffer: [u8; BUFFER_SIZE],
    size: usize
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Command {
    Write,
    Flush
}

pub struct Serial {
    rx: UsbSerialJtagRx<'static, Async>,
    channel_tx: Sender<'static, CriticalSectionRawMutex, Command, CHANNEL_BUFFER>,
}

impl Serial {
    pub async fn init(usb_device: USB_DEVICE<'static>, spawner: &Spawner) {
        let usb_serial = UsbSerialJtag::new(usb_device).into_async();
        let (rx, tx) = usb_serial.split();
        let (sender, receiver) = (CHANNEL.sender(), CHANNEL.receiver());
        spawner.spawn(watch(tx, receiver).unwrap());
        *SERIAL.lock().await = Some(Self { rx, channel_tx: sender });
    }

    pub fn print(&self, bytes: &[u8]) -> Result<(), Error> {
        if bytes.is_empty() {
            return Ok(());
        };
        let mut buffer_data = BUFFER.try_lock().map_err(|_| Error::TryLockError)?;
        let size = buffer_data.size;
        if size + bytes.len() > BUFFER_SIZE {
            return Err(Error::InvalidInput);
        };
        buffer_data.size += bytes.len();
        buffer_data.buffer[size..size + bytes.len()].copy_from_slice(bytes);
        drop(buffer_data);
        self.channel_tx.try_send(Command::Write).map_err(|_| Error::TrySendError)
    } 

    pub fn flush(&self) -> Result<(), Error> {
        self.channel_tx.try_send(Command::Flush).map_err(|_| Error::TrySendError)
    }

    pub fn get() -> Result<MutexGuard<'static, CriticalSectionRawMutex, Option<Serial>>, TryLockError> {
        SERIAL.try_lock()
    }
}

unsafe impl Send for Serial {}
unsafe impl Sync for Serial {}

#[embassy_executor::task]
pub async fn watch(mut tx: UsbSerialJtagTx<'static, Async>, channel_rx: Receiver<'static, CriticalSectionRawMutex, Command, CHANNEL_BUFFER>) -> ! {
    let mut buffer = [0u8; BUFFER_SIZE];
    loop {
        match channel_rx.receive().await {
            Command::Write => {
                let mut buffer_data = BUFFER.lock().await;
                let size = buffer_data.size;
                buffer_data.size = 0;
                buffer.clone_from_slice(&buffer_data.buffer);
                drop(buffer_data);
                if size > BUFFER_SIZE {
                    Write::write(&mut tx, b"\n\r==========\nError: size > BUFFER_SIZE\n==========\n\r").await;
                };
                if size == 0 {
                    continue;
                };
                Write::write(&mut tx, &buffer[..size]).await;
            },
            Command::Flush => { with_timeout(Duration::from_millis(100), Write::flush(&mut tx)).await.ok(); }
        };
    };
}