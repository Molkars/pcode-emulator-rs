use std::path::{Path, PathBuf};

fn main() {
    cxx_build::bridge("src/sleigh/mod.rs")
        .cpp(true)
        .file("src/bridge.cc")
        .files(compilation_files())
        .include(".")
        .flag_if_supported("-std=c++17")
        .warnings(false)
        .compile("sleigh-sleigh");

    println!("cargo:rerun-if-changed=src/sleigh/");
    println!("cargo:rerun-if-changed=src/bridge.cc");
    println!("cargo:rerun-if-changed=include/");
}

fn compilation_files() -> Vec<PathBuf> {
    Path::new("sleigh")
        .read_dir()
        .expect("unable to open sleigh/")
        .flat_map(|file| {
            let file = file.expect("unable to read file");
            let path = file.path();
            path.ends_with(".cc")
                .then(|| path.to_path_buf())
        })
        .collect::<Vec<_>>()
}