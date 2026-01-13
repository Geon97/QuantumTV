use std::fs;
fn main() {
    let ts = fs::read_to_string("../VERSION.txt")
        .expect("VERSION.txt not found")
        .trim()
        .to_string();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", ts);
    tauri_build::build();
}
