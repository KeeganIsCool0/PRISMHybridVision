use std::process::Command;
use std::env;

fn main() {
    // Get the path to the built executable from the build script
    let exe_path = env::var("ATMOS_PANNER_EXE")
        .expect("ATMOS_PANNER_EXE not set - build script should have set this");

    println!("Running atmos-panner from: {}", exe_path);

    // Execute the built C++ program
    let status = Command::new(exe_path)
        .status()
        .expect("failed to execute atmos-panner");

    if !status.success() {
        eprintln!("atmos-panner exited with error: {}", status);
        std::process::exit(1);
    }
}