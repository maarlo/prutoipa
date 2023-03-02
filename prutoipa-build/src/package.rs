use prost_types::FileDescriptorProto;
use std::collections::{btree_map::Entry, BTreeMap};

use crate::{descriptor::Descriptor, error::PrutoipaBuildError, syntax::Syntax};

#[derive(Debug, Clone)]
pub struct Package {
    syntax: Syntax,
    name: String,
    descriptors: BTreeMap<String, Descriptor>,
}

impl Package {
    pub fn new(file: FileDescriptorProto) -> Result<Self, PrutoipaBuildError> {
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

    //
    pub fn get_syntax(&mut self) -> Syntax {
        self.syntax
    }

    pub fn get_name(&mut self) -> String {
        self.name.clone()
    }

    pub fn get_descriptors(&mut self) -> BTreeMap<String, Descriptor> {
        self.descriptors.clone()
    }

    //
    pub fn register_descriptor(
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
