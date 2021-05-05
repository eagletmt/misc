fn main() -> Result<(), Box<dyn std::error::Error>> {
    prost_build::compile_protos(
        &["vendor/grpc/examples/protos/route_guide.proto"],
        &["vendor/grpc/examples/protos"],
    )?;
    Ok(())
}
