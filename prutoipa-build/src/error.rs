#[derive(thiserror::Error, Debug, PartialEq)]
pub enum PrutoipaBuildError {
    #[error("Invalid data: {0}")]
    InvalidData(String),

    #[error("Invalid descriptor set.")]
    InvalidDescriptorSet(#[from] prost::DecodeError),
}
