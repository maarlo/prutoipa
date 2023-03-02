use prost_types::{DescriptorProto, EnumDescriptorProto, FileDescriptorProto, FileDescriptorSet};
use std::collections::BTreeMap;

use crate::{
    descriptor::Descriptor,
    descriptor::{enum_descriptor::EnumDescriptor, message_descriptor::MessageDescriptor},
    error::PrutoipaBuildError,
    package::Package,
};

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

    pub fn get_packages(&self) -> BTreeMap<String, Package> {
        self.packages.clone()
    }

    fn register_file_descriptor_proto(
        &mut self,
        file: FileDescriptorProto,
    ) -> Result<(), PrutoipaBuildError> {
        let mut package = Package::new(file.clone())?;
        let package_name = package.get_name();

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
        let name = descriptor
            .name
            .clone()
            .ok_or(PrutoipaBuildError::InvalidData(
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

        let syntax = package.get_syntax();
        package.register_descriptor(
            name,
            Descriptor::Message(MessageDescriptor::new(syntax, descriptor)?),
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
            Descriptor::Enum(EnumDescriptor::new(descriptor.value)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost_types::{DescriptorProto, EnumDescriptorProto, FileDescriptorProto};

    use crate::{
        descriptor::message_descriptor::field::{Field, FieldType, ScalarType},
        syntax::Syntax,
        tests::{get_fds_encoded, get_file_descriptor_proto},
    };

    #[test]
    fn package_set_simple() {
        fn get_field(descriptors: BTreeMap<String, Descriptor>, name: String) -> Option<Field> {
            if let Descriptor::Message(message_descriptor) = descriptors.get("Person").unwrap() {
                message_descriptor
                    .get_fields()
                    .into_iter()
                    .find(|field| field.get_name() == name)
            } else {
                None
            }
        }

        let fds_encoded = get_fds_encoded(vec![get_file_descriptor_proto()]);

        let mut package_set = PackageSet::default();
        package_set
            .register_file_descriptor_set_encoded(fds_encoded.as_slice())
            .unwrap();

        let mut package = package_set.packages.get("people").unwrap().to_owned();
        let package_name = package.get_name();
        let package_syntax = package.get_syntax();

        assert_eq!(package_name, "people");
        assert_eq!(package_syntax, Syntax::Proto3);

        let field = get_field(package.get_descriptors(), "id".to_string()).unwrap();
        let field_name = field.get_name();
        let field_type = field.get_field_type();

        assert_eq!(field_name, "id");
        assert_eq!(field_type, FieldType::Scalar(ScalarType::I32));

        let field = get_field(package.get_descriptors(), "other_attribute".to_string()).unwrap();
        let field_name = field.get_name();
        let field_type = field.get_field_type();

        assert_eq!(field_name, "other_attribute");
        assert_eq!(field_type, FieldType::Scalar(ScalarType::String));
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

        let expected_err = Some(PrutoipaBuildError::InvalidData(
            "Package 'people' already defined.".to_string(),
        ));

        assert_eq!(format!("{err:?}"), format!("{expected_err:?}"));
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

        let expected_err = Some(PrutoipaBuildError::InvalidData(
            "Expected message name.".to_string(),
        ));

        assert_eq!(format!("{err:?}"), format!("{expected_err:?}"));
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

        let expected_err = Some(PrutoipaBuildError::InvalidData(
            "Expected enum name.".to_string(),
        ));

        assert_eq!(format!("{err:?}"), format!("{expected_err:?}"));
    }
}
