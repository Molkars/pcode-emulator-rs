use std::collections::BTreeMap;
use std::ops::{Index};
use anyhow::{bail, Context};
use hashbrown::{HashMap, HashSet};
use sleigh::{Decompiler, Instruction, PCode, VarnodeData, X86Mode};
use crate::binary::{Binary, Section};
use crate::emulator::{Emulator};

pub struct Machine {
    pub decompiler: Decompiler,

    pub sections: HashSet<String>,
    pub pcodes: HashMap<u64, Vec<PCode>>,
    pub instructions: BTreeMap<u64, Instruction>,
    pub register_names: HashMap<VarnodeData, String>,
    pub named_registers: HashMap<String, VarnodeData>,
}

impl Machine {
    fn load_function(&mut self, binary: &Binary, name: &str) -> anyhow::Result<(u64, u64)> {
        let symbol = binary.symbols
            .get(name)
            .context("unable to find symbol")?;
        println!("loading function: {:X}", symbol.front().unwrap().address);
        assert_eq!(symbol.len(), 1);
        let symbol = symbol.front().unwrap();
        self.load_section(binary, &symbol.section)?;

        println!("loaded function: {} at {:0>8X} with {} bytes", name, symbol.address, symbol.size);
        Ok((symbol.address, symbol.size))
    }

    fn load_section<'a>(&mut self, binary: &'a Binary, name: impl AsRef<str>) -> anyhow::Result<&'a Section> {
        let name = name.as_ref();
        let Some(section) = binary.sections.get(name) else {
            bail!("unknown section");
        };

        if self.sections.contains(name) {
            return Ok(section);
        }

        let start: usize = section.offset.try_into().unwrap();
        let bytes = binary.bytes.index(start..);

        let (_, pcodes) = self.decompiler.translate(bytes, section.address, section.size);
        let (_, instructions) = self.decompiler.disassemble(bytes, section.address, section.size);

        for pcode in pcodes {
            self.pcodes.entry(pcode.address)
                .or_default()
                .push(pcode);
        }
        for instruction in instructions {
            self.instructions.insert(instruction.address, instruction);
        }

        self.sections.insert(name.to_string());
        Ok(section)
    }

    pub fn new(binary: &Binary) -> anyhow::Result<Self> {
        let mut emulator = Machine {
            decompiler: Decompiler::builder().x86(X86Mode::Mode32).build(),
            sections: HashSet::new(),
            pcodes: HashMap::default(),
            instructions: BTreeMap::default(),
            register_names: HashMap::default(),
            named_registers: HashMap::default(),
        };

        for (name, section) in binary.sections.iter() {
            if section.flags.iter().any(|flag| flag == "SHF_EXECINSTR") {
                println!("loading section: {}", name);
                emulator.load_section(binary, name.as_str())?;
            }
        }
        println!("done loading sections");

        emulator.register_names = emulator.decompiler.get_all_registers();
        emulator.named_registers = emulator.register_names.iter()
            .map(|(node, name)| (name.clone(), node.clone()))
            .collect();

        Ok(emulator)
    }

    pub fn emulate(&mut self, binary: &Binary, symbol: &str) -> anyhow::Result<(Emulator, Cursor)> {
        let (address, size) = self.load_function(binary, symbol)?;
        // I don't know the size of instructions so we're going to find the last one
        let end_address = *self.instructions.range(..address + size).next_back().unwrap().0;

        let emulator = Emulator::new(self.register_names.clone());
        println!("emulating {} at {:0>8X} with {} bytes", symbol, address, size);

        // todo: support more architectures
        let ebp = emulator.get_register("EBP")
            .expect("unable to find EBP register");
        emulator.write(ebp, 0u32);

        let esp = emulator.get_register("ESP")
            .expect("unable to find ESP register");
        emulator.write(esp, 0x100_000u32);

        let eip = emulator.get_register("EIP")
            .expect("unable to find EIP register");
        emulator.write(eip, u32::try_from(address).unwrap());

        let cursor = Cursor {
            address,
            index: 0,
            end_address,
        };

        Ok((emulator, cursor))
    }
}

pub struct Cursor {
    pub address: u64,
    pub index: usize,
    pub end_address: u64,
}

impl Cursor {
    pub fn next(&mut self, machine: &Machine) -> Option<PCode> {
        loop {
            let Some(pcodes) = machine.pcodes.get(&self.address) else {
                panic!("no pcodes for address {:0>8X}", self.address);
            };

            if let Some(pcode) = pcodes.get(self.index) {
                self.index += 1;
                if self.index == pcodes.len() && self.address != self.end_address {
                    self.address = *machine.instructions.range(self.address + 1..)
                        .next()
                        .expect("no instruction for pcode")
                        .0;
                    self.index = 0;
                }
                return Some(pcode.clone());
            }

            if self.address == self.end_address {
                return None;
            }

            let (addr, _) = machine.instructions.range(self.address + 1..)
                .next()
                .expect("no instruction for pcode");
            self.address = *addr;
            self.index = 0;
        }
    }

    pub fn set_address(&mut self, address: u64, machine: &Machine) {
        machine.instructions.get(&address)
            .expect("no instruction for pcode");
        self.address = address;
        self.index = 0;
    }
}