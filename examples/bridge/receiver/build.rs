//! Generates the client and server stubs from the shared contract.
//!
//! A build script is fine here and would not be in the published crates: this
//! directory is outside the workspace, so `cargo add rs-teststand` still
//! compiles without running anything.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../proto/rs_teststand_bridge.proto");

    // prost-build shells out to protoc. Point it at a vendored copy rather than
    // requiring one on PATH, so this builds with nothing installed first.
    //
    // SAFETY: a build script is single-threaded at this point, so there is no
    // other thread that could be reading the environment concurrently.
    unsafe {
        std::env::set_var("PROTOC", protoc_bin_vendored::protoc_bin_path()?);
    }

    tonic_prost_build::compile_protos("../proto/rs_teststand_bridge.proto")?;
    Ok(())
}
