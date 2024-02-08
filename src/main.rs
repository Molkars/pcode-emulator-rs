#![allow(dead_code, unused_variables)]

use pcode::binary::Binary;
use pcode::emulator::Emulator;
use crate::cli::{CLI, Command};

mod util;
mod cli;

fn main() -> anyhow::Result<()> {
    // build_binaries();

    let args = <CLI as clap::Parser>::parse();
    match args.command {
        Command::Emulate { binary, .. } => {
            // let content = util::read_file_as_bytes(&binary)
            //     .with_context(|| format!("unable to read {} as bytes", binary.display()))?;
            //
            // let mut decompiler = Decompiler::builder().x86(sleigh::X86Mode::Mode32).build();
            // let base_address = get_base_address(binary.as_path())?;
            //
            // let (n, instructions) = decompiler.disassemble(&content[0x1000..], base_address, 0);
            // println!("read {n} bytes for {} instructions", instructions.len());
            //
            // let path = Path::new(binary.file_name().unwrap()).with_extension("instructions.pcrs");
            // println!("saving instructions to {:?}", path);
            // util::write_to_file(path.as_path(), |f| {
            //     for inst in &instructions {
            //         writeln!(f, "{:0>8X} | ({}) {}", inst.address, inst.mnemonic, inst.body)?;
            //     }
            //     Ok(())
            // })?;
            //
            //
            // let (n, pcodes) = decompiler.translate(&content[0x1000..], base_address, 0);
            // println!("read {n} bytes for {} pcodes", pcodes.len());
            //
            // let path = Path::new(binary.file_name().unwrap())
            //     .with_extension("pcodes.pcrs");
            // println!("saving pcodes to {:?}", path);
            // util::write_to_file(path.as_path(), |f| {
            //     for pcode in &pcodes {
            //         writeln!(f, "{:0>8X} | {:?}", pcode.address, pcode.opcode)?;
            //     }
            //     Ok(())
            // })?;
            //
            // let registers = decompiler.get_all_registers();
            //
            // let symbol_table = SymbolTable::build_symbol_table(binary)?;
            //
            // let pcode_tree = {
            //     let mut out = BTreeMap::new();
            //     for pcode in pcodes {
            //         let entry: &mut LinkedList<_> = out.entry(pcode.address).or_default();
            //         entry.push_back(pcode);
            //     }
            //     out
            // };
            //
            // let mut entry_to_pcodes = HashMap::new();
            // for (entry, info) in symbol_table.iter() {
            //     let lower = Bound::Included(info.address);
            //     let upper = Bound::Included(info.address + info.size);
            //     let mut pcodes = Vec::new();
            //     for (_, pcode_list) in pcode_tree.range((lower, upper)) {
            //         pcodes.extend(pcode_list.iter().cloned());
            //     }
            //     entry_to_pcodes.insert(entry.to_string(), pcodes);
            // }
            //
            // Emulator::emulate(&entry_to_pcodes, "main", &symbol_table, &registers)?;

            let binary = Binary::x86_32(binary)?;
            Emulator::emulate(&binary, "main")?;
        }
    };

    Ok(())
}
