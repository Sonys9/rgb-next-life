#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to send the value")]
    TrySendError,

    #[error("Failed to lock the value")]
    TryLockError,

    #[error("Invalid input")]
    InvalidInput,

    #[error("Failed to erase the storage")]
    StorageErase,

    #[error("Failed to write the storage")]
    StorageWrite,

    #[error("The data is too big")]
    TooBig,

    #[error("Buffer is too small")]
    Buffer,

    #[error("Parse error")]
    Parse,

    #[error("Failed to read the storage")]
    StorageRead,
}
