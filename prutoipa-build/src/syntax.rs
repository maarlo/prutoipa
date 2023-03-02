use crate::error::PrutoipaBuildError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Syntax {
    Proto2,
    Proto3,
}

impl Syntax {
    pub fn get(syntax: Option<&str>) -> Result<Syntax, PrutoipaBuildError> {
        match syntax {
            None | Some("proto2") => Ok(Syntax::Proto2),
            Some("proto3") => Ok(Syntax::Proto3),
            Some(s) => Err(PrutoipaBuildError::InvalidData(format!(
                "Unknown syntax: {s}"
            ))),
        }
    }
}
