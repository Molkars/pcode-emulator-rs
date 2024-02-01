#![allow(dead_code, unused_variables)]

use std::env;
use std::io::Write;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::ops::Index;
use std::path::Path;
use std::process::{Command, exit};
use sleigh::Decompiler;
use crate::command::CommandUtil;
// use crate::sleigh::SleighBridge;

mod command;

mod emulator;

// fn build_binaries() {
//     const CFLAGS: &[&str] = &["example.c"];
//
//     if !Path::new("bin").exists() {
//         std::fs::create_dir("bin").unwrap();
//     }
//
//     if !Path::new("bin/example.x86").exists() {
//         Command::new("clang")
//             .args(["-m32", "-march=x86-32"])
//             .args(["-o", "bin/example.x86-32"])
//             .args(CFLAGS)
//             .output()
//             .unwrap()
//             .expect_success();
//     }
// }

fn usage() {
    eprintln!("usage: pcem-rs <binary> [-offset int] [-limit int] [-base int]");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("    binary : the path to a binary file");
    eprintln!();
    eprintln!("Options:");
    eprintln!("    -offset : the initial offset past the base the address");
    eprintln!("    -limit  : the limit on how many addresses to visit");
    eprintln!("    -base   : the base address of the section");
}

fn main() -> anyhow::Result<()> {
    // build_binaries();

    let args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        usage();
        exit(1);
    }

    let binary = Path::new(args[0].as_str());
    if !binary.exists() {
        eprintln!("<binary> file does not exist!");
        exit(1);
    }

    let mut offset = None;
    let mut limit = None;
    let mut base = None;

    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        match arg {
            "-offset" => {
                let Some(arg) = args.get(i) else {
                    usage();
                    eprintln!();
                    eprintln!("expected value after '-offset'");
                    exit(1);
                };
                let (arg, radix) = arg.strip_prefix("0x")
                    .map(|arg| (arg, 16))
                    .unwrap_or((arg.as_str(), 10));
                offset = Some(u64::from_str_radix(arg, radix)
                    .unwrap_or_else(|e| {
                        eprintln!("invalid value for '-offset': {}", e);
                        exit(1);
                    }));
                i += 2;
            }
            "-limit" => {
                let Some(arg) = args.get(i) else {
                    usage();
                    eprintln!();
                    eprintln!("expected value after '-offset'");
                    exit(1);
                };
                let (arg, radix) = arg.strip_prefix("0x")
                    .map(|arg| (arg, 16))
                    .unwrap_or((arg.as_str(), 10));
                limit = Some(u64::from_str_radix(arg, radix)
                    .unwrap_or_else(|e| {
                        eprintln!("invalid value for '-offset': {}", e);
                        exit(1);
                    }));
                i += 2;
            }
            "-base" => {
                let Some(arg) = args.get(i) else {
                    usage();
                    eprintln!();
                    eprintln!("expected value after '-base'");
                    exit(1);
                };
                let arg = arg.strip_prefix("0x").unwrap_or(arg.as_str());
                base = Some(u64::from_str_radix(arg, 16)
                    .unwrap_or_else(|e| {
                        eprintln!("invalid value for '-offset': {}", e);
                        exit(1);
                    }));
                i += 2;
            }
            _ => {
                usage();
                eprintln!();
                eprintln!("error: unknown option {arg:?}");
                exit(1);
            }
        };
    }

    let content = {
        let mut code = Vec::new();
        let file = match File::open(binary) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("unable to read binary file");
                env::var("DEBUG").map(|_| eprintln!("error: {}", e)).ok();
                exit(1);
            }
        };
        let mut file = BufReader::new(file);
        if let Err(e) = file.read_to_end(&mut code) {
            eprintln!("unable to read binary file");
            env::var("DEBUG").map(|_| eprintln!("error: {}", e)).ok();
            exit(1);
        };
        code
    };

    let mut decompiler = Decompiler::builder()
        .x86(sleigh::X86Mode::Mode32)
        .build();

    // Emulator::emulate(pcodes.as_slice())
    //     .context("unable to emulate pcode!")?;

    Ok(())
}
