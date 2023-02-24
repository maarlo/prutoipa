mod error;
mod package_set;

#[derive(Debug, Default)]
pub struct Builder {
    package_set: package_set::PackageSet,
}

impl Builder {
    /// Create a new `Builder`
    pub fn new() -> Self {
        Self::default()
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
}
