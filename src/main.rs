#![allow(dead_code, unused_variables)]

use anyhow::Context;
use pcode::binary::Binary;
use pcode::emulator::{Emulator, PcodeControl};
use crate::cli::{CLI, Command};

mod util;
mod cli;

fn main() -> anyhow::Result<()> {
    // build_binaries();

    let args = <CLI as clap::Parser>::parse();
    match args.command {
        Command::Emulate { binary, .. } => {
            let binary = Binary::x86_32(binary)?;
            let mut emulator = Emulator::new(&binary)?;

            let mut state = emulator.emulate("main")?;
            println!("------");
            while let Some((i, pcode)) = state.next() {
                let instruction = state.emulator.instructions.get(&pcode.address)
                    .expect("no instruction for pcode");
                println!("emulating {:0>8X}.{:0>2X} {: <20?} - ({}) {}", pcode.address, i, pcode.opcode, instruction.mnemonic, instruction.body);
                let control = state.emulator.emulate_one(pcode)
                    .context("emulation failed")?;
                println!();

                match control {
                    PcodeControl::Branch(target) => {
                        state.set_address(target);
                        // if end_address == target {
                        //     break;
                        // }
                        //
                        // // todo: find end of basic block?
                        // pcode_group_iter = emulator.pcodes.range(target..);
                        // let (new_addr, new_vec) = pcode_group_iter.next().expect("no more pcodes!");
                        // addr = *new_addr;
                        // pcode_iter = new_vec.iter().enumerate();
                    }
                    PcodeControl::Continue => {}
                };
            }

            let eax = state.emulator.named_registers.get("EAX").expect("no eax");
            let value = state.emulator.get_int(eax);
            println!("eax: {}", value);
        }
    };

    Ok(())
}
