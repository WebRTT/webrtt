use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodecError {
    #[error("failed to encode message: {0}")]
    Encode(rmp_serde::encode::Error),

    #[error("failed to decode message: {0}")]
    Decode(rmp_serde::decode::Error),
}
