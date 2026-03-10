use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend/");
    println!("cargo:rerun-if-changed=build.rs");

    let status = Command::new("npm")
        .args(["install"])
        .current_dir("frontend")
        .status()
        .expect("failed to run npm install");

    if !status.success() {
        panic!("npm install failed");
    }

    let status = Command::new("npm")
        .args(["run", "build"])
        .current_dir("frontend")
        .status()
        .expect("failed to run npm build");

    if !status.success() {
        panic!("npm build failed");
    }
}
