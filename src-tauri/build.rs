use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target = env::var("TARGET").unwrap();
    
    // Only embed on Windows
    if !target.contains("windows") {
        return;
    }

    // Create binaries directory
    let bin_dir = out_dir.join("binaries");
    fs::create_dir_all(&bin_dir).unwrap();

    // Copy binaries if they exist in src-tauri/binaries
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source_bin_dir = manifest_dir.join("binaries");
    
    if source_bin_dir.exists() {
        for entry in fs::read_dir(&source_bin_dir).unwrap() {
            let entry = entry.unwrap();
            let dest = bin_dir.join(entry.file_name());
            fs::copy(entry.path(), &dest).unwrap();
        }
    }

    // Tell cargo to rerun if binaries change
    println!("cargo:rerun-if-changed=binaries/");
    println!("cargo:rustc-env=BINARIES_DIR={}", bin_dir.display());
}