use std::env;
use std::path::PathBuf;

// This build script looks for the correct build dependencies (static and dynamic libraries) and
// does two things:
// 1. Copy all .dll files to the manifest directory.
// 2. 
// copies all .dll files from the build directory
fn main() {
    let target = env::var("TARGET").unwrap();

    if target.contains("pc-windows") {
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

        let mut dependencies_dir = manifest_dir.clone();
        dependencies_dir.push("build");

        let mut lib_dir = dependencies_dir.clone();
        let mut dll_dir = dependencies_dir.clone();

        if target.contains("msvc") {
            lib_dir.push("msvc");
            dll_dir.push("msvc");
        } else {
            panic!("Target libraries not found: {}", target);
        }

        lib_dir.push("lib");
        dll_dir.push("dll");

        if target.contains("x86_64") {
            lib_dir.push("x64");
            dll_dir.push("x64");
        } else {
            lib_dir.push("x86");
            dll_dir.push("x86");
        }

        println!("cargo:rustc-link-search=all={}", lib_dir.display());

        for entry in std::fs::read_dir(dll_dir).expect("Can't read DLL dir")  {
            let entry_path = entry.expect("Invalid fs entry").path();
            let file_name_result = entry_path.file_name();
            let mut new_file_path = manifest_dir.clone();
            if let Some(file_name) = file_name_result {
                let file_name = file_name.to_str().unwrap();
                if file_name.ends_with(".dll") {
                    new_file_path.push(file_name);
                    std::fs::copy(&entry_path, new_file_path.as_path()).expect("Can't copy from DLL dir");
                }
            }
        }
    }
}
