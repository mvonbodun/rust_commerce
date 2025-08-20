fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[allow(dead_code)]");
    config.compile_protos(&["proto/inventory.proto", "proto/code.proto"], &["proto/"])?;
    Ok(())
}
