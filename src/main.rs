#![allow(dead_code, unused_variables)]

use std::collections::HashMap;
use std::io::Write;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::path::Path;
use std::process::Command;
use sleigh::{ArchState, Decompiler, DecompilerBuilder, X86Mode};
use crate::command::CommandUtil;
// use crate::sleigh::SleighBridge;

mod command;

mod emulator;

fn build_binaries() {
    const CFLAGS: &[&str] = &["-c", "example.c"];

    if !Path::new("bin").exists() {
        std::fs::create_dir("bin").unwrap();
    }

    if !Path::new("bin/example.x86").exists() {
        Command::new("clang")
            .args(["-m32", "-march=x86-32"])
            .args(["-o", "bin/example.x86-32"])
            .args(CFLAGS)
            .output()
            .unwrap()
            .expect_success();
    }
}

fn main() -> anyhow::Result<()> {
    build_binaries();

    let mut code = Vec::new();
    let mut file = BufReader::new(File::open("example.bin")
        .expect("unable to open `example`"));
    file.read_to_end(&mut code)
        .expect("unable to read file");
    drop(file);
    println!("read {len} bytes", len = code.len());

    let mut decompiler = Decompiler::builder()
        .x86(X86Mode::Mode32)
        .build();


    println!("hi!");
    let (n, pcodes) = decompiler.translate(code.as_slice(), 0x0000);
    println!("read {n} {}", pcodes.len());
    // for (addr, group) in &pcodes.iter().group_by(|item| item.address) {
    //     print!("{addr:0>4} | ");
    //     for (i, pcode) in group.enumerate() {
    //         if i > 0 {
    //             print!(", ");
    //         }
    //         print!("P({:?} {})", pcode.opcode, pcode.vars.len());
    //     }
    //     println!();
    // }

    for code in pcodes.iter() {
        println!("PCode: {}, {:?}", code.address, code.opcode);
    }
    println!("done with {}", pcodes.len());

    let (len, insts) = decompiler.disassemble(code.as_slice(), 0x0000);
    println!("instructions: {}", len);

    let outfile = File::create("../sleigh-rs.txt").unwrap();
    let mut outfile = BufWriter::new(outfile);
    for inst in insts.iter() {
        writeln!(&mut outfile, "{:0>8X} | ({}) {}", inst.address, inst.mnemonic, inst.body)
            .unwrap();
    }
    drop(outfile);

    // Emulator::emulate(pcodes.as_slice())
    //     .context("unable to emulate pcode!")?;

    Ok(())
}
