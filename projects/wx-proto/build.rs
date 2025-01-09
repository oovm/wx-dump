use std::io::Result;
fn main() -> Result<()> {
    let file_descriptors = protox::compile(["proto/msg.proto", "proto/roomdata.proto"], ["proto"]).unwrap();
    prost_build::compile_fds(file_descriptors)?;
    Ok(())
}
