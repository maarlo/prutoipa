#[derive(thiserror::Error, Debug)]
pub enum PrutoipaBuildError {
    #[error("{0} is not implemented yet.")]
    NotImplementedYet(String),

    #[error("Output dir not set.")]
    OutputDirNotSet,

    #[error("Invalid data: {0}.")]
    InvalidData(String),

    #[error("Invalid descriptor set.")]
    InvalidDescriptorSet(#[from] prost::DecodeError),

    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
