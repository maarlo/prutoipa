use prost_types::EnumValueDescriptorProto;

#[derive(Debug, Clone)]
pub struct EnumDescriptor {
    values: Vec<EnumValueDescriptorProto>,
}

#[derive(Clone)]
pub struct EnumValue {
    pub name: String,
    pub number: i32,
}

impl EnumDescriptor {
    pub fn new(values: Vec<EnumValueDescriptorProto>) -> Self {
        Self { values }
    }

    pub fn get_values(&self) -> Vec<EnumValue> {
        self.values
            .clone()
            .into_iter()
            .map(|evdp| EnumValue {
                name: evdp.name().to_string(),
                number: evdp.number(),
            })
            .collect::<Vec<EnumValue>>()
    }
}
