fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Skip protobuf compilation - use pre-generated code instead
    // This avoids the dependency on external protoc compiler
    println!("cargo:rerun-if-changed=proto/frame.proto");
    Ok(())
}