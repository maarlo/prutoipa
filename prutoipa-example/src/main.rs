use std::{env, fs, io::Error, path::PathBuf};

fn build_protos(
    descriptors_file_path: &PathBuf,
    protos_folder: PathBuf,
    proto_files: Vec<PathBuf>,
) -> Result<(), Error> {
    prost_build::Config::new()
        // Allow proto3 optional
        .protoc_arg("--experimental_allow_proto3_optional")
        // Save descriptors to file
        .file_descriptor_set_path(descriptors_file_path)
        // Generate prost structs
        .compile_protos(&proto_files, &[protos_folder])
}

fn main() {
    // Proto files location
    let protos_folder = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("protos");
    let proto_files = vec![protos_folder.join("person.proto")];

    // File where proto descriptors will be saved
    let descriptors_file_path =
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("proto_descriptor.bin");

    // Build protos
    build_protos(&descriptors_file_path, protos_folder, proto_files).unwrap();

    // Build utoipa code
    let descriptors = fs::read(descriptors_file_path).unwrap();
    prutoipa_build::Builder::new()
        .register_descriptors(&descriptors)
        .unwrap()
        .build()
        .unwrap();
}
