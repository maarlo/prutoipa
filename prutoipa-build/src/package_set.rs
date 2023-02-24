use prost_types::{
    DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
    FileDescriptorProto, FileDescriptorSet, MessageOptions, OneofDescriptorProto,
};
use std::collections::{btree_map::Entry, BTreeMap};

use crate::error::PrutoipaBuildError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Syntax {
    Proto2,
    Proto3,
}

impl Syntax {
    fn get(syntax: Option<&str>) -> Result<Syntax, PrutoipaBuildError> {
        match syntax {
            None | Some("proto2") => Ok(Syntax::Proto2),
            Some("proto3") => Ok(Syntax::Proto3),
            Some(s) => Err(PrutoipaBuildError::InvalidData(format!(
                "Unknown syntax: {s}"
            ))),
        }
    }
}

#[derive(Debug, Clone)]
struct Package {
    syntax: Syntax,
    name: String,
    descriptors: BTreeMap<String, Descriptor>,
}

impl Package {
    fn new(file: FileDescriptorProto) -> Result<Self, PrutoipaBuildError> {
        let syntax = Syntax::get(file.syntax.as_deref())?;
        let name = file.package.ok_or(PrutoipaBuildError::InvalidData(
            "Expected package name.".to_string(),
        ))?;

        Ok(Self {
            syntax,
            name,
            descriptors: BTreeMap::<String, Descriptor>::new(),
        })
    }

    fn register_descriptor(
        &mut self,
        name: String,
        descriptor: Descriptor,
    ) -> Result<(), PrutoipaBuildError> {
        match self.descriptors.entry(name) {
            Entry::Occupied(o) => Err(PrutoipaBuildError::InvalidData(format!(
                "Descriptor '{}' registered more than once at the same package.",
                o.key()
            ))),
            Entry::Vacant(v) => {
                v.insert(descriptor);
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PackageSet {
    packages: BTreeMap<String, Package>,
}

impl PackageSet {
    pub fn register_file_descriptor_set_encoded(
        &mut self,
        fds_encoded: &[u8],
    ) -> Result<(), PrutoipaBuildError> {
        let file_descriptor_set: FileDescriptorSet = prost::Message::decode(fds_encoded)
            .map_err(|e| PrutoipaBuildError::InvalidDescriptorSet(e))?;

        self.register_file_descriptor_set(file_descriptor_set)
    }

    pub fn register_file_descriptor_set(
        &mut self,
        file_descriptor_set: FileDescriptorSet,
    ) -> Result<(), PrutoipaBuildError> {
        file_descriptor_set
            .file
            .into_iter()
            .map(|file| self.register_file_descriptor_proto(file))
            .collect()
    }

    fn register_file_descriptor_proto(
        &mut self,
        file: FileDescriptorProto,
    ) -> Result<(), PrutoipaBuildError> {
        let mut package = Package::new(file.clone())?;
        let package_name = package.name.clone();

        match self.packages.contains_key(&package_name) {
            true => Err(PrutoipaBuildError::InvalidData(format!(
                "Package '{}' already defined.",
                package_name
            ))),
            false => {
                file.message_type
                    .into_iter()
                    .map(|descriptor| self.register_message(&mut package, descriptor))
                    .collect::<Result<(), PrutoipaBuildError>>()?;

                file.enum_type
                    .into_iter()
                    .map(|descriptor| self.register_enum(&mut package, descriptor))
                    .collect::<Result<(), PrutoipaBuildError>>()?;

                self.packages.insert(package_name, package);

                Ok(())
            }
        }
    }

    fn register_message(
        &mut self,
        package: &mut Package,
        descriptor: DescriptorProto,
    ) -> Result<(), PrutoipaBuildError> {
        let name = descriptor.name.ok_or(PrutoipaBuildError::InvalidData(
            "Expected message name.".to_string(),
        ))?;

        // TODO: Check how to do it.
        // descriptor
        //     .nested_type
        //     .into_iter()
        //     .map(|child_descriptor| self.register_message(syntax, package, child_descriptor))
        //     .collect::<Result<(), PrutoipaBuildError>>()?;

        // descriptor
        //     .enum_type
        //     .into_iter()
        //     .map(|child_descriptor| self.register_enum(package, child_descriptor))
        //     .collect::<Result<(), PrutoipaBuildError>>()?;

        package.register_descriptor(
            name,
            Descriptor::Message(MessageDescriptor {
                syntax: package.syntax,
                package: package.name.clone(),
                options: descriptor.options,
                one_of: descriptor.oneof_decl,
                fields: descriptor.field,
            }),
        )
    }

    fn register_enum(
        &mut self,
        package: &mut Package,
        descriptor: EnumDescriptorProto,
    ) -> Result<(), PrutoipaBuildError> {
        let name = descriptor.name.ok_or(PrutoipaBuildError::InvalidData(
            "Expected enum name.".to_string(),
        ))?;

        package.register_descriptor(
            name,
            Descriptor::Enum(EnumDescriptor {
                values: descriptor.value,
            }),
        )
    }
}

#[derive(Debug, Clone)]
pub enum Descriptor {
    Message(MessageDescriptor),
    Enum(EnumDescriptor),
}

#[derive(Debug, Clone)]
pub struct MessageDescriptor {
    pub syntax: Syntax,
    pub package: String,
    pub options: Option<MessageOptions>,
    pub one_of: Vec<OneofDescriptorProto>,
    pub fields: Vec<FieldDescriptorProto>,
}

#[derive(Debug, Clone)]
pub struct EnumDescriptor {
    pub values: Vec<EnumValueDescriptorProto>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost::Message;
    use prost_types::{
        field_descriptor_proto::Type, DescriptorProto, EnumDescriptorProto,
        EnumValueDescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet,
    };

    fn get_file_descriptor_proto() -> FileDescriptorProto {
        FileDescriptorProto {
            syntax: Some("proto3".to_string()),
            package: Some("people".to_string()),
            name: Some("person.proto".to_string()),
            enum_type: vec![EnumDescriptorProto {
                name: Some("GENDER".to_string()),
                value: vec![
                    EnumValueDescriptorProto {
                        name: Some("MALE".to_string()),
                        number: Some(0),
                        ..Default::default()
                    },
                    EnumValueDescriptorProto {
                        name: Some("FEMALE".to_string()),
                        number: Some(1),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            message_type: vec![DescriptorProto {
                name: Some("Person".to_string()),
                field: vec![FieldDescriptorProto {
                    r#type: Some(Type::Int32.into()),
                    name: Some("id".to_string()),
                    number: Some(1),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn get_fds_encoded(files: Vec<FileDescriptorProto>) -> Vec<u8> {
        let mut fds_encoded = Vec::new();

        FileDescriptorSet { file: files }
            .encode(&mut fds_encoded)
            .unwrap();

        fds_encoded
    }

    #[test]
    fn package_simple() {
        let fds_encoded = get_fds_encoded(vec![get_file_descriptor_proto()]);

        let mut package_set = PackageSet::default();
        package_set
            .register_file_descriptor_set_encoded(fds_encoded.as_slice())
            .unwrap();

        let package_name = package_set.packages.get("people").unwrap().name.clone();
        let package_syntax = package_set.packages.get("people").unwrap().syntax;

        assert_eq!(package_name, "people");
        assert_eq!(package_syntax, Syntax::Proto3);
    }

    #[test]
    fn package_defined_twice() {
        let fds_encoded = get_fds_encoded(vec![
            get_file_descriptor_proto(),
            get_file_descriptor_proto(),
        ]);

        let mut package_set = PackageSet::default();
        let err = package_set
            .register_file_descriptor_set_encoded(fds_encoded.as_slice())
            .err();

        assert_eq!(
            err,
            Some(PrutoipaBuildError::InvalidData(
                "Package 'people' already defined.".to_string()
            ))
        );
    }

    #[test]
    fn message_descriptor_without_name() {
        let file_descriptor_proto = get_file_descriptor_proto();
        let fds_encoded = get_fds_encoded(vec![FileDescriptorProto {
            message_type: vec![DescriptorProto {
                name: None,
                ..file_descriptor_proto.message_type[0].clone()
            }],
            ..file_descriptor_proto
        }]);

        let mut package_set = PackageSet::default();
        let err = package_set
            .register_file_descriptor_set_encoded(fds_encoded.as_slice())
            .err();

        assert_eq!(
            err,
            Some(PrutoipaBuildError::InvalidData(
                "Expected message name.".to_string()
            ))
        );
    }

    #[test]
    fn enum_descriptor_without_name() {
        let file_descriptor_proto = get_file_descriptor_proto();
        let fds_encoded = get_fds_encoded(vec![FileDescriptorProto {
            enum_type: vec![EnumDescriptorProto {
                name: None,
                ..file_descriptor_proto.enum_type[0].clone()
            }],
            ..file_descriptor_proto
        }]);

        let mut package_set = PackageSet::default();
        let err = package_set
            .register_file_descriptor_set_encoded(fds_encoded.as_slice())
            .err();

        assert_eq!(
            err,
            Some(PrutoipaBuildError::InvalidData(
                "Expected enum name.".to_string()
            ))
        );
    }
}
