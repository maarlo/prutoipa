use prost_types::EnumValueDescriptorProto;

#[derive(Debug, Clone)]
pub struct EnumDescriptor {
    values: Vec<EnumValueDescriptorProto>,
}

impl EnumDescriptor {
    pub fn new(values: Vec<EnumValueDescriptorProto>) -> Self {
        Self { values }
    }
}
