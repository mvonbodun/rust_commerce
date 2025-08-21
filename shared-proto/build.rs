fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = prost_build::Config::new();

    // Allow dead_code since these are shared types that may not all be used everywhere
    config.type_attribute(".", "#[allow(dead_code)]");
    // Only add serde for types that don't use google.protobuf.Any
    config.type_attribute(
        "common.Code",
        "#[derive(serde::Serialize, serde::Deserialize)]",
    );

    config.compile_protos(
        &["proto/common/code.proto", "proto/common/status.proto"],
        &["proto/"],
    )?;

    Ok(())
}
