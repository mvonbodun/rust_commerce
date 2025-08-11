fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(&[
        "proto/catalog.proto", 
        "proto/category.proto",
        "proto/code.proto"
    ], &["proto/"])?;
    Ok(())
}
