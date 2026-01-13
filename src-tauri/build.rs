use std::fs;

fn main() {
    // 从 tauri.conf.json 读取版本号并设置为编译时环境变量
    if let Ok(json_str) = fs::read_to_string("tauri.conf.json") {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&json_str) {
            if let Some(version) = json.get("version").and_then(|v| v.as_str()) {
                println!("cargo:rustc-env=APP_VERSION={}", version);
            }
        }
    }

    // 告诉 Cargo 当这些文件改变时重新构建
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=../VERSION.txt");

    tauri_build::build();
}
