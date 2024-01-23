use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::process::Command;
use sleigh::Decompiler;
use crate::command::CommandUtil;

mod command;

mod emulator;

fn main() {
    if !Path::new("example.executable").exists() {
        Command::new("clang")
            .args(["-m32", "-march=x86-64"])
            .args(["-o", "example.executable"])
            .args(["-c", "example.c"])
            .output()
            .unwrap()
            .expect_success();
    }

    let mut decompiler = Decompiler::builder()
        .x86(sleigh::X86Mode::Mode32)
        .build();

    let mut code = Vec::new();
    let mut file = BufReader::new(File::open("example.executable")
        .expect("unable to open `example`"));
    file.read_to_end(&mut code)
        .expect("unable to read file");

    let (_, pcodes) = decompiler.translate(code.as_slice(), 0x1000);
    for pcode in pcodes {
        println!("address: {:?}", pcode.address);
        println!("opcode: {:?}", pcode.opcode);
        for varnode in pcode.vars {
            println!("  varnode: {varnode:?}");
        }
        println!("outvar: {:?}", pcode.outvar);

        println!();
    }

    let (len, insts) = decompiler.disassemble(code.as_slice(), 0x1000);
    println!("{} {:?}", len, insts);
}
