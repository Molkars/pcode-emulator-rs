#![allow(dead_code, unused_variables)]

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::process::Command;
use sleigh::Decompiler;
use crate::command::CommandUtil;

mod command;

mod emulator;

mod arbint;

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

#[allow(unused)]
mod _arbitrary_int {
    fn slice_add(
        lhs: &[u8],
        rhs: &[u8]
    ) -> (Vec<u8>, bool) {
        assert_eq!(lhs.len(), rhs.len());
        let mut out = vec![0; lhs.len()];
        let mut overflowed = false;
        for (i, (a, b)) in lhs.iter().rev().zip(rhs.iter().rev()).enumerate() {
            let (result, did_overflow) = a.overflowing_add(*b);

            let (result, did_overflow) = if overflowed {
                let (result, interior_overflow) = result.overflowing_add(1);
                (result, interior_overflow | did_overflow)
            } else {
                (result, did_overflow)
            };
            overflowed = did_overflow;
            println!("result, overflow = {:0>8b}, {}", result, did_overflow);

            out[lhs.len() - i - 1] = result;
        }
        (out, overflowed)
    }

    #[test]
    fn test_slice_add() {
        let a = &[0b1000_1000, 0b0000_1000];
        let b = a;
        let (result, overflowed) = slice_add(a, b);

        print!("   ");
        for item in a.iter() {
            print!("{item:0>8b}");
        }
        println!();

        print!(" + ");
        for item in b.iter() {
            print!("{item:0>8b}");
        }
        println!();

        print!(" = ");
        for item in result.iter() {
            print!("{item:0>8b}");
        }
        print!(" (overflow={})", overflowed);
        println!();
    }
}