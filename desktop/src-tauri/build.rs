use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=lib/");

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = manifest_dir.join("lib");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();

    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    match target_os.as_str() {
        "linux" => {
            println!("cargo:rustc-link-lib=dylib=vosk");
            println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/lib");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
        }
        "macos" => {
            println!("cargo:rustc-link-lib=dylib=vosk");
            println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/lib");
            println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/../Resources/lib");
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", lib_dir.display());
        }
        "windows" => {
            println!("cargo:rustc-link-lib=dylib=vosk");
        }
        other => panic!("[Peppa] Unsupported target OS: {other}"),
    }

    tauri_build::build()
}
