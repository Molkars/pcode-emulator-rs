#![allow(dead_code, unused_variables)]

use std::collections::{BTreeMap, LinkedList};
use std::env;
use std::io::Write;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read};
use std::ops::Bound;
use std::path::Path;
use std::process::{Command, exit};
use hashbrown::HashMap;
use sleigh::Decompiler;
use crate::emulator::Emulator;
use crate::symbol_table::SymbolTable;
// use crate::sleigh::SleighBridge;

mod command;
mod symbol_table;
mod emulator;

fn usage() {
    eprintln!("usage: pcem-rs <binary> [func]");
    eprintln!();
    eprintln!("Arguments:");
    eprintln!("    binary : the path to a binary file");
    eprintln!("    [func] : the name of a function symbol in the binary");
    eprintln!();
    eprintln!("Options:");
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

    // let symbol_table = OnceCell::new();
    // let get_symbol_table = || {
    //     symbol_table.get_or_init(move || build_symbol_table(binary))
    // };
    //
    // let func = args.get(1).map(|symbol| {
    //     let Some(symbol_info) = get_symbol_table().get(symbol) else {
    //         eprintln!("unknown symbol: {}", symbol);
    //         exit(1);
    //     };
    //     symbol_info
    // });

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

    let base_address = get_base_address(binary);

    let (n, instructions) = decompiler.disassemble(&content[0x1000..], base_address, 0);
    println!("read {n} bytes for {} instructions", instructions.len());

    let path = Path::new(binary.file_name().unwrap())
        .with_extension("pcode");
    println!("saving instructions to {:?}", path);
    let file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("unable to save instructions");
            env::var("DEBUG").map(|_| eprintln!("error: {e}")).ok();
            exit(1);
        }
    };
    let mut file = BufWriter::new(file);
    for (i, instruction) in instructions.iter().enumerate() {
        if i > 0 {
            writeln!(&mut file).unwrap();
        }
        write!(&mut file, "{:0>8X} | ({}) {}", instruction.address, instruction.mnemonic, instruction.body)
            .unwrap();
    }

    let (n, pcodes) = decompiler.translate(&content[0x1000..], base_address, 0);
    println!("read {n} bytes for {} pcodes", pcodes.len());
    let path = Path::new(binary.file_name().unwrap())
        .with_extension("pcodes");
    println!("saving instructions to {:?}", path);
    let file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("unable to save pcodes");
            env::var("DEBUG").map(|_| eprintln!("error: {e}")).ok();
            exit(1);
        }
    };
    let mut file = BufWriter::new(file);
    for (i, pcode) in pcodes.iter().enumerate() {
        if i > 0 {
            writeln!(&mut file).unwrap();
        }
        write!(&mut file, "{:0>8X} | {:?}", pcode.address, pcode.opcode).unwrap();
    }

    let symbol_table = SymbolTable::build_symbol_table(binary)?;
    let main = symbol_table.get("main")
        .expect("unable to locate main")
        .address;

    let pcode_tree = {
        let mut out = BTreeMap::new();
        for pcode in pcodes {
            let entry: &mut LinkedList<_> = out.entry(pcode.address).or_default();
            entry.push_back(pcode);
        }
        out
    };

    let mut organized_pcodes = HashMap::new();
    for (entry, info) in symbol_table.iter() {
        let lower = Bound::Included(info.address);
        let upper = Bound::Included(info.address + info.size);
        let mut pcodes = Vec::new();
        for (_, pcode_list) in pcode_tree.range((lower, upper)) {
            pcodes.extend(pcode_list.iter().cloned());
        }
        organized_pcodes.insert(entry.to_string(), pcodes);
    }

    Emulator::emulate(&organized_pcodes, "main", &symbol_table)?;

    Ok(())
}

fn with_open(path: impl AsRef<Path>, f: impl FnOnce(&mut BufWriter<File>) -> std::io::Result<()>)
             -> std::io::Result<()>
{
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    f(&mut writer)
}

fn get_base_address(path: impl AsRef<Path>) -> u64 {
    let symbol_table = Command::new("llvm-objdump")
        .arg("--x86-asm-syntax=intel")
        .arg("-d")
        .arg(path.as_ref())
        .output();
    let symbol_table = match symbol_table {
        Ok(output) => {
            String::from_utf8(output.stdout)
                .unwrap_or_else(|e| {
                    eprintln!("unable to parse objdump output");
                    exit(1);
                })
        }
        Err(e) => {
            eprintln!("unable to run objdump on the specified binary: {}", e);
            exit(1);
        }
    };
    symbol_table.lines()
        .nth(5)
        .map(|line| {
            println!("base_addr_line: {line:?}");
            let (addr, _) = line.split_once(' ').unwrap();
            u64::from_str_radix(addr, 16).unwrap()
        })
        .unwrap()
}