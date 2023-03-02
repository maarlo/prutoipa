pub mod enum_descriptor;
pub mod message_descriptor;

use crate::descriptor::{enum_descriptor::EnumDescriptor, message_descriptor::MessageDescriptor};

#[derive(Debug, Clone)]
pub enum Descriptor {
    Message(MessageDescriptor),
    Enum(EnumDescriptor),
}
