fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../fiveg_proto/nudm-sdm.proto")?;
    Ok(())
}
