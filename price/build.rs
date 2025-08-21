fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[allow(dead_code)]");

    // Include the shared proto path
    config.compile_protos(
        &["proto/offer.proto"],
        &["proto/", "../shared-proto/proto/"],
    )?;
    Ok(())
}
