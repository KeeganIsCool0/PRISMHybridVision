// build.rs
use std::process::Command;
use std::env;
use std::fs;

fn main() {
    // Create build directory
    let build_dir = "Firmware/build";
    fs::create_dir_all(build_dir).expect("Failed to create build directory");

    // Run cmake with testing disabled
    let cmake_status = Command::new("cmake")
        .current_dir(build_dir)
        .arg("..")
        .arg("-DBUILD_TESTING=OFF")
        .status()
        .expect("Failed to run cmake");

    if !cmake_status.success() {
        panic!("cmake failed");
    }

    // Run make
    let make_status = Command::new("make")
        .current_dir(build_dir)
        .status()
        .expect("Failed to run make");

    if !make_status.success() {
        panic!("make failed");
    }

    // Tell Rust code where to find the executable
    let exe_path = format!("{}/atmos-panner", build_dir);
    println!("cargo:rustc-env=ATMOS_PANNER_EXE={}", exe_path);
}