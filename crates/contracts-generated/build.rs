//! Build-time Protobuf compilation using a pinned vendored `protoc` binary.

use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

fn collect_proto_files(directory: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_proto_files(&path, files)?;
        } else if path
            .extension()
            .is_some_and(|extension| extension == "proto")
        {
            files.push(path);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let include_root = manifest.join("../../contracts/proto");
    println!("cargo:rerun-if-changed={}", include_root.display());

    let mut protos = Vec::new();
    collect_proto_files(&include_root, &mut protos)?;
    protos.sort();
    if protos.is_empty() {
        return Err(format!("no .proto files found under {}", include_root.display()).into());
    }

    let protoc_path = protoc_bin_vendored::protoc_bin_path()?;
    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc_path);
    config.file_descriptor_set_path(
        PathBuf::from(env::var("OUT_DIR")?).join("wildfire_descriptor.bin"),
    );
    config.compile_protos(&protos, &[include_root])?;
    Ok(())
}
