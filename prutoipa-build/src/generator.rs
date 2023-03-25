pub mod enumeration;
pub mod message;

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy)]
struct Indent(usize);

impl Display for Indent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.0 {
            write!(f, "    ")?;
        }
        Ok(())
    }
}
