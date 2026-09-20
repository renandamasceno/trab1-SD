use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("PROTOC").is_err() && Path::new("/opt/homebrew/bin/protoc").exists() {
        unsafe {
            env::set_var("PROTOC", "/opt/homebrew/bin/protoc");
        }
    }
    prost_build::compile_protos(&["proto/calculator.proto"], &["proto/"])?;
    Ok(())
}
