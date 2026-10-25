#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to send the value")]
    TrySendError,

    #[error("Failed to lock the value")]
    TryLockError,

    #[error("Invalid input")]
    InvalidInput
}