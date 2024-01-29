#![allow(dead_code, unused_variables)]

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::process::Command;
use sleigh::Decompiler;
use crate::command::CommandUtil;

mod command;

mod emulator;

fn build_binaries() {
    const CFLAGS: &[&str] = &["-c", "example.c"];

    if !Path::new("bin").exists() {
        std::fs::create_dir("bin").unwrap();
    }

    if !Path::new("bin/example.x86").exists() {
        Command::new("clang")
            .args(["-m32", "-march=x86-64"])
            .args(["-o", "bin/example.x86"])
            .args(CFLAGS)
            .output()
            .unwrap()
            .expect_success();
    }
}

fn main() -> anyhow::Result<()> {
    build_binaries();

    let mut decompiler = Decompiler::builder()
        .x86(sleigh::X86Mode::Mode32)
        .build();

    let mut code = Vec::new();
    let mut file = BufReader::new(File::open("example.executable")
        .expect("unable to open `example`"));
    file.read_to_end(&mut code)
        .expect("unable to read file");
    println!("read {len} bytes", len=code.len());

    // let (_, pcodes) = sleigh.translate(code.as_slice(), 0x0000);
    // for pcode in pcodes.iter() {
    //     println!("address: {:?}", pcode.address);
    //     println!("opcode: {:?}", pcode.opcode);
    //     for varnode in pcode.vars.iter() {
    //         println!("  varnode: {varnode:?}");
    //     }
    //     println!("outvar: {:?}", pcode.outvar);
    //
    //     println!();
    // }

    let (len, insts) = decompiler.disassemble(code.as_slice(), 0x0000);
    println!("{} {:?}", len, insts);

    // Emulator::emulate(pcodes.as_slice())
    //     .context("unable to emulate pcode!")?;

    Ok(())
}
