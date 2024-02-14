#![allow(dead_code, unused_variables)]

use anyhow::Context;
use pcode::binary::Binary;
use pcode::emulator::{Machine, PCodeControl};
use crate::cli::{CLI, Command};

mod util;
mod cli;

fn main() {
    run().unwrap()
}

fn run() -> anyhow::Result<()> {
    let args = <CLI as clap::Parser>::parse();
    match args.command {
        Command::Emulate { binary, .. } => {
            let binary = Binary::x86_32(binary)?;
            let mut machine = Machine::new(&binary)?;

            let mut emulator = machine.emulate("main")?;

            println!("-=- Emulating -=-");
            while let Some((i, pcode)) = emulator.next() {
                let instruction = emulator.emulator.instructions.get(&pcode.address)
                    .expect("no instruction for pcode");
                println!("emulating {:0>8X}.{:0>2X} {: <20?} - ({}) {}", pcode.address, i, pcode.opcode, instruction.mnemonic, instruction.body);
                let control = emulator.emulate_one(pcode)
                    .context("emulation failed")?;
                println!();

                match control {
                    PCodeControl::Branch(target) => {
                        emulator.set_address(target);
                    }
                    PCodeControl::Continue => {}
                };
            }

            println!("-=- Done -=-");
            let eax = emulator.emulator.named_registers.get("EAX").expect("no eax");
            let value = emulator.read::<i32>(eax);
            println!("$eax = {}", value);
        }
    };

    Ok(())
}