use std::io::Result;

fn main() -> Result<()> {
    let mut config = prost_build::Config::new();
    config.type_attribute(
        "UpdateConfig",
        "#[derive(serde::Serialize, serde::Deserialize)]",
    );
    config.compile_protos(&["../messages.proto"], &["../", "."])?;
    Ok(())
}
