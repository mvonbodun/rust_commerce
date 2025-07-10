fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(&["proto/orders.proto", "proto/code.proto"], &["proto/"])?;
    Ok(())
}
