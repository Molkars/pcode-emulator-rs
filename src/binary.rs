use std::collections::BTreeMap;
use std::rc::Rc;
use std::path::{Path};
use std::process::{Command, exit};
use std::str::FromStr;
use anyhow::{anyhow, Context};
use hashbrown::HashMap;
use sleigh::{Decompiler, Instruction, PCode, VarnodeData, X86Mode};
use crate::symbol_table::SymbolInfo;
use crate::util::{self, ExecUtil};
use std::io::Write;

#[derive(Debug)]
pub struct FunctionSymbol {
    pub section: String,
    pub address: usize,
    pub size: usize,
    pub pcode_index: usize,
    pub pcode_count: usize,
    pub instruction_index: usize,
    pub instruction_count: usize,
}

pub struct Binary {
    /// binary bytecode
    pub bytecode: Rc<[u8]>,
    /// base address of the binary
    pub base_address: u64,
    /// x86 instructions
    pub instructions: Rc<[Instruction]>,
    /// pcode instructions
    pub pcodes: Rc<[PCode]>,
    /// function name to function symbol
    pub function_table: BTreeMap<String, FunctionSymbol>,
    /// register location to register name
    pub registers: HashMap<VarnodeData, String>,
    pub address_to_pcode_index: HashMap<u64, usize>,
    pub address_to_instruction_index: HashMap<u64, usize>,
}

impl Binary {
    pub fn x86_32(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let bytecode: Rc<[u8]> = util::read_file_as_bytes(path)?.into();

        let mut decompiler = Decompiler::builder()
            .x86(X86Mode::Mode32)
            .build();

        let base_address = get_base_address(path)
            .context("unable to get base address of binary")?;

        let (_, pcodes) = decompiler.translate(&bytecode[0x1000..], base_address, 0);
        let pcodes_path = Path::new(path.file_name().unwrap()).with_extension("pcodes.pcrs");
        println!("saving pcodes to {:?}", pcodes_path);
        util::write_to_file(pcodes_path.as_path(), |f| {
            for pcode in &pcodes {
                writeln!(f, "{:0>8X} | {:?}", pcode.address, pcode.opcode)?;
            }
            Ok(())
        })?;

        let (_, instructions) = decompiler.disassemble(&bytecode[0x1000..], base_address, 0);
        let instructions_path = Path::new(path.file_name().unwrap()).with_extension("instructions.pcrs");
        println!("saving instructions to {:?}", instructions_path);
        util::write_to_file(instructions_path.as_path(), |f| {
            for inst in &instructions {
                writeln!(f, "{:0>8X} | ({}) {}", inst.address, inst.mnemonic, inst.body)?;
            }
            Ok(())
        })?;

        let mut function_table = build_symbol_table(path)
            .context("unable to build symbol table")?;
        let function_table_path = Path::new(path.file_name().unwrap()).with_extension("symbols.pcrs");
        println!("saving function table to {:?}", function_table_path);
        util::write_to_file(function_table_path.as_path(), |f| {
            for (name, symbol) in &function_table {
                writeln!(f, "{:0>8X} | {} | {} | {}", symbol.address, symbol.size, symbol.section, name)?;
            }
            Ok(())
        })?;

        let mut addr_to_pcode_index = HashMap::new();
        for (i, pcode) in pcodes.iter().enumerate() {
            if !addr_to_pcode_index.contains_key(&pcode.address) {
                addr_to_pcode_index.insert(pcode.address, i);
            }
        }

        let mut addr_to_instruction_index = HashMap::new();
        for (i, inst) in instructions.iter().enumerate() {
            if !addr_to_instruction_index.contains_key(&inst.address) {
                addr_to_instruction_index.insert(inst.address, i);
            }
        }

        for (name, symbol) in function_table.iter_mut() {
            let addr = symbol.address as u64;
            let Some(&pcode_index) = addr_to_pcode_index.get(&addr) else {
                eprintln!("unable to find pcode index for function {}", name);
                continue;
            };
            let end_addr = addr + symbol.size as u64;
            let pcode_count = pcodes[pcode_index..]
                .iter()
                .take_while(|pcode| pcode.address < end_addr)
                .count();

            let instruction_index = *addr_to_instruction_index.get(&addr)
                .context("unable to find instruction index")?;
            let instruction_count = instructions[instruction_index..]
                .iter()
                .take_while(|inst| inst.address < end_addr)
                .count();

            symbol.instruction_index = instruction_index;
            symbol.instruction_count = instruction_count;
            symbol.pcode_index = pcode_index;
            symbol.pcode_count = pcode_count;
        }

        let registers = decompiler.get_all_registers();

        Ok(Self {
            bytecode,
            base_address,
            instructions: instructions.into(),
            pcodes: pcodes.into(),
            function_table,
            registers,
            address_to_pcode_index: addr_to_pcode_index,
            address_to_instruction_index: addr_to_instruction_index,
        })
    }

    pub fn function_pcode(&self, func: &str) -> anyhow::Result<&[PCode]> {
        let symbol = self.function_table.get(func)
            .context("function not found")?;
        Ok(&self.pcodes[symbol.pcode_index..symbol.pcode_index + symbol.pcode_count])
    }
}

fn get_base_address(path: impl AsRef<Path>) -> anyhow::Result<u64> {
    let symbol_table = util::exec("llvm-objdump")
        .arg("--x86-asm-syntax=intel")
        .arg("-d")
        .arg(path.as_ref())
        .exec_and_get_stdout_as_string()
        .context("unable to get objdump")?;
    symbol_table.lines()
        .nth(5)
        .ok_or_else(|| anyhow!("objdump presented invalid output: expected the first section on line 5"))
        .and_then(|line| {
            let (addr, _) = line.split_once(' ')
                .ok_or_else(|| anyhow!("unable to splice address"))?;
            u64::from_str_radix(addr, 16)
                .context("unable to parse address")
        })
}

fn build_symbol_table(path: impl AsRef<Path>) -> anyhow::Result<BTreeMap<String, FunctionSymbol>> {
    let symbol_table = Command::new("llvm-objdump")
        .arg("-t")
        .arg(path.as_ref())
        .output();
    let symbol_table = match symbol_table {
        Ok(output) => {
            String::from_utf8(output.stdout)
                .unwrap_or_else(|_| {
                    eprintln!("unable to parse objdump output");
                    exit(1);
                })
        }
        Err(e) => {
            eprintln!("unable to run objdump on the specified binary: {}", e);
            exit(1);
        }
    };
    symbol_table
        .lines()
        .skip(4)
        .filter_map(|line| {
            Symbol::from_str(line)
                .map(|symbol| {
                    if !symbol.flags.contains('F') {
                        return None;
                    }
                    Some((symbol.name, FunctionSymbol {
                        section: symbol.section,
                        address: symbol.address as usize,
                        size: symbol.size as usize,
                        pcode_index: 0,
                        pcode_count: 0,
                        instruction_index: 0,
                        instruction_count: 0,
                    }))
                })
                .transpose()
        })
        .collect::<anyhow::Result<_>>()
}

struct Symbol {
    name: String,
    section: String,
    address: u64,
    size: u64,
    flags: String,
}

impl FromStr for Symbol {
    type Err = anyhow::Error;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        if line.contains('\n') {
            return Err(anyhow!("line contains newline"));
        }

        let (address, rest) = line.split_once(' ')
            .context("unable to splice base address")?;
        let address = u64::from_str_radix(address, 16)
            .context("unable to parse base address")?;

        let flags = rest[..7].to_string();
        let rest = &rest[8..];
        let (section, rest) = rest.split_once('\t')
            .context("unable to slice section")?;
        let (size, name) = rest.split_once(' ')
            .context("unable to slice entry size")?;
        let size = u64::from_str_radix(size, 16)
            .context("unable to parse entry size")?;

        Ok(Self {
            name: name.to_string(),
            section: section.to_string(),
            address,
            size,
            flags: flags.to_string(),
        })
    }
}