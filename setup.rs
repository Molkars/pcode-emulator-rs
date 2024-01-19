use std::path::Path;
use std::process::Command;

macro_rules! error {
    ($($t:tt)*) => {{
        eprintln!($($t)*);
        std::process::exit(1);
    }}
}

fn main() {
    Command::new("git")
        .args(["clone", "https://github.com/NationalSecurityAgency/ghidra"])
        .status()
        .expect("unable to checkout ghidra!");

    let library_root = Path::new("ghidra/Ghidra/Features/Decompiler/src/decompile/cpp");
    if !dir_exists(library_root) {
        error!(
            "ghidra did not install correctly, expected decompiler at {}",
            library_root.display()
        );
    }
}

#[inline]
fn dir_exists(path: impl AsRef<Path>) -> bool {
    let path = path.as_ref();
    path.exists() && path.metadata().ok().is_some_and(|meta| meta.is_dir())
}
