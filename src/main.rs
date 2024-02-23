#![allow(dead_code, unused_variables)]

use anyhow::{anyhow, Context};
use pcode::binary::Binary;
use pcode::emulator::{Machine, PCodeControl};
use crate::cli::{CLI, Command};

mod util;
mod cli;

#[cfg(feature = "ui")]
mod ui;

fn main() {
    run().unwrap()
}

fn run() -> anyhow::Result<()> {
    let args = <CLI as clap::Parser>::parse();
    match args.command {
        Command::Emulate { binary, .. } => {
            let binary = Binary::x86_32(binary)?;
            let mut machine = Machine::new(&binary)?;

            let (emulator, mut cursor) = machine.emulate(&binary, "main")?;

            let pcode = cursor.next(&machine).unwrap();

            println!("-=- Emulating -=-");
            while let Some(pcode) = cursor.next(&machine) {
                let instruction = machine.instructions.get(&pcode.address)
                    .expect("no instruction for pcode");
                println!("emulating {:0>8X}.{:0>2X} {: <20?} - ({}) {}", pcode.address, cursor.index, pcode.opcode, instruction.mnemonic, instruction.body);
                let control = emulator.emulate_one(&pcode)
                    .context("emulation failed")?;
                println!();

                match control {
                    PCodeControl::Branch(target) => {
                        if target != cursor.end_address {
                            cursor.set_address(target, &machine);
                        } else {
                            break;
                        }
                    }
                    PCodeControl::Continue => {}
                };
            }

            println!("-=- Done -=-");
            let eax = emulator.get_register("eax")
                .expect("unable to find eax register");
            let value = emulator.read::<i32>(eax);
            println!("$eax = {}", value);
        }
    };

    Ok(())
}