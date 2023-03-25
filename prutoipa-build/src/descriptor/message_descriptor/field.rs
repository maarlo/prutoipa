use heck::ToSnakeCase;
use prost_types::{
    field_descriptor_proto::{Label, Type},
    FieldDescriptorProto,
};

use crate::{error::PrutoipaBuildError, syntax::Syntax};

#[derive(Debug, Clone)]
pub struct Field {
    name: String,
    field_modifier: FieldModifier,
    field_type: FieldType,
}

impl Field {
    pub fn new(
        syntax: &Syntax,
        field_descriptor_proto: &FieldDescriptorProto,
    ) -> Result<Self, PrutoipaBuildError> {
        let name = field_descriptor_proto
            .name
            .clone()
            .ok_or(PrutoipaBuildError::InvalidData(
                "Expected field to have name".to_string(),
            ))?;
        let field_type = Self::get_type(field_descriptor_proto)?;
        let field_modifier = Self::get_modifier(syntax, field_descriptor_proto, &field_type)?;

        Ok(Self {
            name,
            field_modifier,
            field_type,
        })
    }

    //
    pub fn get_name(&self) -> String {
        self.name.to_snake_case()
    }

    pub fn get_field_modifier(&self) -> FieldModifier {
        self.field_modifier.clone()
    }

    pub fn get_field_type(&self) -> FieldType {
        self.field_type.clone()
    }

    //
    fn get_type(field: &FieldDescriptorProto) -> Result<FieldType, PrutoipaBuildError> {
        match field.type_name.as_ref() {
            Some(type_name) => {
                let splitted_type_name = type_name.split(".").collect::<Vec<&str>>();
                if splitted_type_name.len() == 3 {
                    Ok(FieldType::Object {
                        package: splitted_type_name[1].to_string(),
                        descriptor: splitted_type_name[2].to_string(),
                    })
                } else {
                    Err(PrutoipaBuildError::InvalidData(format!(
                        "The object {type_name} is not valid."
                    )))
                }
            }
            None => {
                let prost_type = Type::from_i32(
                    field
                        .r#type
                        .ok_or(PrutoipaBuildError::InvalidData("Expected type".to_string()))?,
                )
                .ok_or(PrutoipaBuildError::InvalidData(
                    "Expected valid type.".to_string(),
                ))?;

                let scalar = match prost_type {
                    Type::Double => ScalarType::F64,
                    Type::Float => ScalarType::F32,
                    Type::Int64 | Type::Sfixed64 | Type::Sint64 => ScalarType::I64,
                    Type::Int32 | Type::Sfixed32 | Type::Sint32 => ScalarType::I32,
                    Type::Uint64 | Type::Fixed64 => ScalarType::U64,
                    Type::Uint32 | Type::Fixed32 => ScalarType::U32,
                    Type::Bool => ScalarType::Bool,
                    Type::String => ScalarType::String,
                    Type::Bytes => ScalarType::Bytes,
                    Type::Message | Type::Enum | Type::Group => panic!("no type name specified"),
                };

                Ok(FieldType::Scalar(scalar))
            }
        }
    }

    fn get_modifier(
        syntax: &Syntax,
        field: &FieldDescriptorProto,
        field_type: &FieldType,
    ) -> Result<FieldModifier, PrutoipaBuildError> {
        let label = Label::from_i32(field.label.ok_or(PrutoipaBuildError::InvalidData(
            "Expcted label.".to_string(),
        ))?)
        .ok_or(PrutoipaBuildError::InvalidData(
            "Exptected valid label.".to_string(),
        ))?;

        if field.proto3_optional.unwrap_or(false) {
            assert_eq!(label, Label::Optional);
            Ok(FieldModifier::Optional)
        } else if field.oneof_index.is_some() {
            assert_eq!(label, Label::Optional);
            Ok(FieldModifier::Optional)
        } else {
            match label {
                Label::Optional => match syntax {
                    Syntax::Proto2 => Ok(FieldModifier::Optional),
                    Syntax::Proto3 => match field_type {
                        FieldType::Scalar(_) => Ok(FieldModifier::Required),
                        FieldType::Object {
                            package: _,
                            descriptor: _,
                        } => Ok(FieldModifier::Optional),
                    },
                },
                Label::Required => Ok(FieldModifier::Required),
                Label::Repeated => Ok(FieldModifier::Repeated),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Scalar(ScalarType),
    Object { package: String, descriptor: String },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScalarType {
    F64,
    F32,
    I32,
    I64,
    U32,
    U64,
    Bool,
    String,
    Bytes,
}

impl ScalarType {
    pub fn get_utoipa_type(&self) -> &'static str {
        match self {
            Self::String => "String",
            Self::I32 | Self::I64 | Self::U32 | Self::U64 => "Integer",
            Self::F64 | Self::F32 => "Number",
            Self::Bool => "Boolean",
            _ => todo!(
                "{}",
                PrutoipaBuildError::NotImplementedYet("Bytes".to_string())
            ),
        }
    }

    pub fn get_utoipa_format(&self) -> Option<&'static str> {
        match self {
            Self::I32 | Self::U32 => Some("Int32"),
            Self::I64 | Self::U64 => Some("Int64"),
            Self::F32 => Some("Float"),
            Self::F64 => Some("Double"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldModifier {
    Required,
    Optional,
    Repeated,
}
