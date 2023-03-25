mod descriptor;
mod error;
mod generator;
mod package;
mod package_set;
mod syntax;

use std::{
    io::{BufWriter, Write},
    path::PathBuf,
};

use descriptor::Descriptor;
use error::PrutoipaBuildError;
use generator::{enumeration::generate_enum, message::generate_message};
use package::Package;
use package_set::PackageSet;

#[derive(Debug, Default)]
pub struct Builder {
    out_dir: Option<PathBuf>,
    package_set: PackageSet,
    generate_enum_values: bool,
}

impl Builder {
    /// Create a new `Builder`
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the output directory where generated Rust files will be written.
    ///
    /// If unset, defaults to the `OUT_DIR` environment variable. `OUT_DIR` is set by Cargo when
    /// executing build scripts, so `out_dir` typically does not need to be configured.
    pub fn out_dir<P>(&mut self, path: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        self.out_dir = Some(path.into());
        self
    }

    pub fn generate_enum_values(&mut self) -> &mut Self {
        self.generate_enum_values = true;
        self
    }

    /// Register an encoded `FileDescriptorSet` with this `Builder`
    pub fn register_descriptors(
        &mut self,
        fds_encoded: &[u8],
    ) -> Result<&mut Self, error::PrutoipaBuildError> {
        self.package_set
            .register_file_descriptor_set_encoded(fds_encoded)?;

        Ok(self)
    }

    fn get_out_dir(&self) -> Result<PathBuf, PrutoipaBuildError> {
        if let Some(out_dir) = self.out_dir.clone() {
            Ok(out_dir)
        } else {
            Ok(std::env::var_os("OUT_DIR")
                .ok_or(PrutoipaBuildError::OutputDirNotSet)?
                .into())
        }
    }

    pub fn build(&mut self) -> Result<(), PrutoipaBuildError> {
        let mut output = self.get_out_dir()?;
        output.push("DUMMY_FILENAME");

        self.package_set
            .get_packages()
            .into_iter()
            .map(|(_, mut package)| self.generate(&mut output, &mut package))
            .collect::<Result<Vec<()>, PrutoipaBuildError>>()?;

        Ok(())
    }

    fn generate(
        &self,
        output: &mut PathBuf,
        package: &mut Package,
    ) -> Result<(), PrutoipaBuildError> {
        output.set_file_name(format!("{}.utoipa.rs", package.get_name()));

        let file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&output)?;

        let mut writer = BufWriter::new(file);

        let generator_result = package
            .get_descriptors()
            .into_iter()
            .map(|(descriptor_name, descriptor)| match descriptor {
                Descriptor::Message(message) => {
                    generate_message(&mut writer, package.get_name(), descriptor_name, message)
                }
                Descriptor::Enum(enum_descriptor) => generate_enum(
                    &mut writer,
                    package.get_name(),
                    descriptor_name,
                    enum_descriptor,
                    self.generate_enum_values,
                ),
            })
            .collect::<Result<Vec<()>, PrutoipaBuildError>>();

        writer.flush()?;

        match generator_result {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    use prost::Message;
    use prost_types::{
        field_descriptor_proto::{Label, Type},
        DescriptorProto, EnumDescriptorProto, EnumValueDescriptorProto, FieldDescriptorProto,
        FileDescriptorProto, FileDescriptorSet,
    };

    pub fn get_file_descriptor_proto() -> FileDescriptorProto {
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
                field: vec![
                    FieldDescriptorProto {
                        r#type: Some(Type::Int32.into()),
                        name: Some("id".to_string()),
                        number: Some(1),
                        label: Some(Label::Optional.into()),
                        ..Default::default()
                    },
                    FieldDescriptorProto {
                        r#type: Some(Type::String.into()),
                        name: Some("otherAttribute".to_string()),
                        number: Some(2),
                        label: Some(Label::Required.into()),
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    pub fn get_fds_encoded(files: Vec<FileDescriptorProto>) -> Vec<u8> {
        let mut fds_encoded = Vec::new();

        FileDescriptorSet { file: files }
            .encode(&mut fds_encoded)
            .unwrap();

        fds_encoded
    }
}
