pub mod field;

use prost_types::DescriptorProto;

use crate::{
    descriptor::message_descriptor::field::Field, error::PrutoipaBuildError, syntax::Syntax,
};

#[derive(Debug, Clone)]
pub struct MessageDescriptor {
    fields: Vec<Field>,
}

impl MessageDescriptor {
    pub fn new(syntax: Syntax, descriptor: DescriptorProto) -> Result<Self, PrutoipaBuildError> {
        let mut fields: Vec<Field> = Vec::new();

        for field_descriptor_proto in &descriptor.field {
            let field = Field::new(&syntax, field_descriptor_proto)?;

            // Treat synthetic one-of as normal
            let proto3_optional = field_descriptor_proto.proto3_optional.unwrap_or(false);
            match (field_descriptor_proto.oneof_index, proto3_optional) {
                (Some(_idx), false) => {
                    println!(
                        "{}",
                        PrutoipaBuildError::NotImplementedYet("One of".to_string())
                    )
                }
                _ => fields.push(field),
            }
        }

        Ok(Self { fields })
    }

    //
    pub fn get_fields(&self) -> Vec<Field> {
        self.fields.clone()
    }
}
