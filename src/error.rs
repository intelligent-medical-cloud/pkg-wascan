use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum Error {
    #[error("ERR_NO_WINDOW")]
    WindowNotFound,

    #[error("ERR_NO_DOCUMENT")]
    DocumentNotFound,

    #[error("ERR_NO_TRIGGER_BUTTONS")]
    TriggerButtonsNotFound,

    #[error("ERR_NO_FILE_SELECTED")]
    NoFileSelected,

    #[error("ERR_INVALID_MIME")]
    InvalidMime,

    #[error("ERR_NOT_IMAGE_FILE")]
    NotImageFile,

    #[error("ERR_NOT_DETECTED")]
    NotDetected,

    #[error("ERR_INTERNAL")]
    Internal,
}
