use std::collections::BTreeMap;
use std::ops::Index;
use anyhow::{bail, Context};
use hashbrown::{HashMap, HashSet};
use sleigh::{Decompiler, Instruction, PCode, VarnodeData, X86Mode};
use crate::binary::{Binary, Section};
use crate::emulator::{Emulator};

pub struct Machine<'a> {
    pub binary: &'a Binary,
    pub decompiler: Decompiler,

    pub sections: HashSet<String>,
    pub pcodes: BTreeMap<u64, Vec<PCode>>,
    pub instructions: BTreeMap<u64, Instruction>,
    pub register_names: HashMap<VarnodeData, String>,
    pub named_registers: HashMap<String, VarnodeData>,
}

impl<'a> Machine<'a> {
    pub fn load_function(&mut self, name: &str) -> anyhow::Result<(u64, u64)> {
        let symbol = self.binary.symbols
            .get(name)
            .context("unable to find symbol")?;
        println!("loading function: {:X}", symbol.front().unwrap().address);
        assert_eq!(symbol.len(), 1);
        let symbol = symbol.front().unwrap();
        self.load_section(&symbol.section)?;

        println!("loaded function: {} at {:0>8X} with {} bytes", name, symbol.address, symbol.size);
        Ok((symbol.address, symbol.size))
    }

    fn load_section(&mut self, name: impl AsRef<str>) -> anyhow::Result<&Section> {
        let name = name.as_ref();
        let Some(section) = self.binary.sections.get(name) else {
            bail!("unknown section");
        };

        if self.sections.contains(name) {
            return Ok(section);
        }

        let start: usize = section.offset.try_into().unwrap();
        let bytes = self.binary.bytes.index(start..);

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

    pub fn new(binary: &'a Binary) -> anyhow::Result<Self> {
        let mut emulator = Machine {
            binary,
            decompiler: Decompiler::builder().x86(X86Mode::Mode32).build(),
            sections: HashSet::new(),
            pcodes: BTreeMap::default(),
            instructions: BTreeMap::default(),
            register_names: HashMap::default(),
            named_registers: HashMap::default(),
        };

        for (name, section) in binary.sections.iter() {
            if section.flags.iter().any(|flag| flag == "SHF_EXECINSTR") {
                println!("loading section: {}", name);
                emulator.load_section(name.as_str())?;
            }
        }
        println!("done loading sections");

        emulator.register_names = emulator.decompiler.get_all_registers();
        emulator.named_registers = emulator.register_names.iter()
            .map(|(node, name)| (name.clone(), node.clone()))
            .collect();

        Ok(emulator)
    }

    pub fn emulate(&mut self, symbol: &str) -> anyhow::Result<Emulator<'_, 'a>> {
        let (address, size) = self.load_function(symbol)?;
        // I don't know the size of instructions so we're going to find the last one
        let end_address = *self.instructions.range(..address + size).next_back().unwrap().0;

        let emulator = Emulator::new(self, address, end_address);
        println!("emulating {} at {:0>8X} with {} bytes", symbol, address, size);

        // todo: support more architectures
        let ebp = emulator.get_register("EBP")
            .expect("unable to find EBP register");
        emulator.write(ebp, 0u32);

        let esp = emulator.get_register("ESP")
            .expect("unable to find ESP register");
        emulator.write(esp, 0xFFFF_CBB8_u32);

        let eip = emulator.get_register("EIP")
            .expect("unable to find EIP register");
        emulator.write(eip, u32::try_from(emulator.address).unwrap());

        Ok(emulator)
    }
}
