use std::cell::{Ref, RefCell, RefMut};
use std::collections::{btree_map, BTreeMap};
use std::env::var;
use std::mem::MaybeUninit;
use std::ops::{Add, Deref, Index, IndexMut, Range, Sub};
use anyhow::{anyhow, bail, Context};
use hashbrown::{HashMap, HashSet};
use itertools::Itertools;
use num::{BigInt, BigUint, CheckedSub, ToPrimitive, Zero};
use num::bigint::Sign;
use num::traits::{FromBytes, ToBytes};
use sleigh::{AddrSpace, Decompiler, Instruction, Opcode, PCode, SpaceType, VarnodeData, X86Mode};
use sleigh_sys::ffi;
use crate::binary::{Binary, Section};
use crate::inspect;
use crate::symbol_table::SymbolTable;

#[derive(Default, Debug)]
struct Space {
    big_endian: bool,
    inner: RefCell<BTreeMap<u64, u8>>,
    buffer: RefCell<Vec<u8>>,
}

enum PcodeControl {
    Branch(u64),
    Continue,
}

impl Space {
    pub fn new(big_endian: bool) -> Self {
        Self {
            big_endian,
            inner: RefCell::default(),
            buffer: RefCell::new(vec![0; 4]),
        }
    }

    pub fn get_bytes(&self, addr: u64, size: u64) -> Ref<[u8]> {
        let mut inner = self.inner.borrow_mut();
        let mut buffer = self.buffer.borrow_mut();
        buffer.resize(size as usize, 0u8); // fill the rest with 0
        // let start = addr - addr % self.word_size; // align to word boundary
        let start = addr;
        let end = start + size;

        let mut last_key = start;
        for (key, value) in inner.range(start..end) {
            buffer[0..(key - last_key) as usize].fill(0u8);
            buffer[(key - start) as usize] = *value;
            last_key = key + 1;
        }
        buffer.resize(size as usize, 0u8); // fill the rest with 0
        buffer[(last_key - start) as usize..size as usize].fill(0u8);
        drop(buffer);
        Ref::map(self.buffer.borrow(), |vec| &vec[..size as usize])
    }

    pub fn set_bytes(&self, addr: u64, bytes: &[u8]) {
        let mut inner = self.inner.borrow_mut();
        let start = addr;
        let end = start + bytes.len() as u64;
        for (i, byte) in bytes.iter().enumerate() {
            inner.insert(start + i as u64, *byte);
        }
    }
}

pub struct Emulator<'a> {
    binary: &'a Binary,
    decompiler: Decompiler,

    unique_space: Space,
    register_space: Space,
    ram_space: Space,
    const_space: Space,

    sections: HashSet<String>,
    pcodes: BTreeMap<u64, Vec<PCode>>,
    instructions: BTreeMap<u64, Instruction>,
    register_names: HashMap<VarnodeData, String>,
    named_registers: HashMap<String, VarnodeData>,
}

impl<'a> Emulator<'a> {
    fn load_function(&mut self, name: &str) -> anyhow::Result<(u64, u64)> {
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

    pub fn emulate(binary: &Binary, symbol: &str) -> anyhow::Result<()> {
        let mut emulator = Emulator {
            binary,
            decompiler: Decompiler::builder().x86(X86Mode::Mode32).build(),
            ram_space: Space::new(false),
            register_space: Space::new(false),
            unique_space: Space::new(false),
            const_space: Space::new(false),
            sections: HashSet::new(),
            pcodes: BTreeMap::default(),
            instructions: BTreeMap::default(),
            register_names: HashMap::default(),
            named_registers: HashMap::default(),
        };

        let (address, size) = emulator.load_function(symbol)?;
        let end_address = *emulator.instructions.range(..address + size).next_back().unwrap().0;
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

        const INIT_STACK_SIZE: usize = 0x100_000;

        let ebp = emulator.named_registers.get("EBP")
            .expect("unable to find EBP register");
        emulator.write_uint(ebp, &BigUint::from(INIT_STACK_SIZE as u64));

        let esp = emulator.named_registers.get("ESP")
            .expect("unable to find ESP register");
        emulator.write_uint(esp, &BigUint::from(INIT_STACK_SIZE as u64));

        let eip = emulator.named_registers.get("EIP")
            .expect("unable to find EIP register");
        emulator.write_uint(eip, &BigUint::from(address));

        // main arguments
        let ebx = emulator.named_registers.get("EBX")
            .expect("unable to find EBX register");

        println!("emulating {} at {:0>8X} with {} bytes", symbol, address, size);
        let mut pcode_group_iter = emulator.pcodes.range(address..address + size);
        let (mut addr, mut pcode_iter) = {
            let (mut addr, mut vec) = pcode_group_iter.next().expect("no pcodes!");
            let mut pcode_iter = vec.iter().enumerate();
            (*addr, pcode_iter)
        };
        loop {
            let Some((i, pcode)) = pcode_iter.next() else {
                if addr == end_address {
                    break;
                }
                let (new_addr, new_vec) = pcode_group_iter.next().expect("no more pcodes!");
                addr = *new_addr;
                pcode_iter = new_vec.iter().enumerate();
                continue;
            };

            let instruction = emulator.instructions.get(&pcode.address)
                .expect("no instruction for pcode");
            println!("emulating {:0>8X}.{:0>2X} {: <20?} - ({}) {}", pcode.address, i, pcode.opcode, instruction.mnemonic, instruction.body);
            let control = emulator.emulate_one(pcode)
                .context("emulation failed")?;
            println!();

            match control {
                PcodeControl::Branch(target) => {
                    // todo: find end of basic block?
                    pcode_group_iter = emulator.pcodes.range(target..);
                    let (new_addr, new_vec) = pcode_group_iter.next().expect("no more pcodes!");
                    addr = *new_addr;
                    pcode_iter = new_vec.iter().enumerate();
                }
                PcodeControl::Continue => {}
            };
        }

        Ok(())
    }

    #[inline]
    fn get_bytes(&self, node: &VarnodeData) -> Ref<[u8]> {
        let bytes = self.get_space(node)
            .unwrap()
            .get_bytes(node.offset, node.size.into());
        println!("  read {:X?} from {}", bytes, self.nameof(node));
        bytes
    }

    fn set_bytes(&self, node: &VarnodeData, bytes: &[u8]) {
        if matches!(node.space.type_, SpaceType::Constant) {
            panic!("cannot write to constant space");
        }
        println!("  wrote {:X?} to {}", bytes, self.nameof(node));
        self.get_space(node)
            .unwrap()
            .set_bytes(node.offset, bytes);
    }

    #[inline]
    fn write_int(&self, node: &VarnodeData, value: &BigInt) {
        println!("  wrote {:X?} to {}", value, self.nameof(node));
        let mut bytes = vec![0; node.size as usize];
        if node.space.is_big_endian {
            to_int_of_size_be(value, bytes.as_mut());
        } else {
            to_int_of_size_le(value, bytes.as_mut());
        }
        self.set_bytes(node, &bytes);
    }

    #[inline]
    fn write_uint(&self, node: &VarnodeData, value: &BigUint) {
        println!("  wrote {:X?} to {}", value, self.nameof(node));
        let mut bytes = vec![0; node.size as usize];
        if node.space.is_big_endian {
            let be = value.to_bytes_be();
            if be.len() > bytes.len() {
                let split = be.len() - bytes.len();
                bytes.copy_from_slice(&be[split..]);
            } else {
                let split = bytes.len() - be.len();
                bytes[split..].copy_from_slice(&be);
                bytes[..split].fill(0);
            }
        } else {
            let le = value.to_bytes_le();
            if le.len() > bytes.len() {
                let split = bytes.len();
                bytes.copy_from_slice(&le[..split]);
            } else {
                let split = le.len();
                bytes[..split].copy_from_slice(&le);
                bytes[split..].fill(0);
            }
        }
        self.set_bytes(node, &bytes);
    }

    #[inline]
    fn get_space(&self, node: &VarnodeData) -> anyhow::Result<&Space> {
        match node.space.name.as_str() {
            "unique" => Ok(&self.unique_space),
            "register" => Ok(&self.register_space),
            "ram" => Ok(&self.ram_space),
            "const" => Ok(&self.const_space),
            _ => bail!("unsupported space type: {:?}", node.space.name),
        }
    }

    fn nameof(&self, node: &VarnodeData) -> String {
        self.register_names.get(node)
            .cloned()
            .unwrap_or_else(|| format!("{}:{:X}+{}", node.space.name, node.offset, node.size))
    }

    fn emulate_one(
        &self,
        pcode: &PCode,
    ) -> anyhow::Result<PcodeControl> {
        println!("  {:?} : {} -> {}", pcode.opcode,
                 pcode.vars.iter().map(|node| self.nameof(node)).join(", "),
                 pcode.outvar.as_ref().map(|node| self.nameof(node)).unwrap_or("!".to_string()));
        let control = match pcode.opcode {
            Opcode::Copy => {
                let [input0] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                let output = pcode.outvar.as_ref().unwrap();

                assert_eq!(input0.size, output.size, "input and output must have the same size");
                let value = self.get_uint(input0);
                self.write_uint(output, &value);

                PcodeControl::Continue
            }
            Opcode::IntSub => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_int(input0);
                let right = self.get_int(input1);
                let result = left.sub(&right);
                self.write_int(output, &result);
                PcodeControl::Continue
            }
            Opcode::Store => {
                let [input0, input1, input2] = pcode.vars.as_slice() else {
                    bail!("expected 3 inputs");
                };

                let space = self.get_space_from_const(input0)?;
                let offset = self.get_uint(input1).to_u64()
                    .expect("offset must fit in u64");
                let offset = offset * u64::from(space.wordsize); // offset to bytes
                let value = self.get_uint(input2);

                let varnode = VarnodeData { space, offset, size: input2.size };
                self.write_uint(&varnode, &value);

                PcodeControl::Continue
            }
            Opcode::IntSBorrow => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_int(input0);
                let right = self.get_int(input1);
                let result = left - right;
                let overflow = result.bits() > input0.size as u64;
                self.write_bool(output, overflow);
                PcodeControl::Continue
            }
            Opcode::IntLess => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_uint(input0);
                let right = self.get_uint(input1);
                self.write_bool(output, left < right);
                PcodeControl::Continue
            }
            Opcode::IntSLess => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_int(input0);
                let right = self.get_int(input1);
                self.write_bool(output, left < right);
                PcodeControl::Continue
            }
            Opcode::IntEqual => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_uint(input0);
                let right = self.get_uint(input1);
                assert_eq!(output.size, 1, "output must be 1 byte");
                self.write_bool(output, left == right);
                PcodeControl::Continue
            }
            Opcode::IntAnd => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_uint(input0);
                let right = self.get_uint(input1);
                let result = left & right;
                self.write_uint(output, &result);
                PcodeControl::Continue
            }
            Opcode::PopCount => {
                let [input0] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                let output = pcode.outvar.as_ref().unwrap();

                let value = self.get_uint(input0);
                let result = value.count_ones();
                self.write_uint(output, &BigUint::from(result));
                PcodeControl::Continue
            }
            Opcode::Branch => {
                let [addr] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                PcodeControl::Branch(addr.offset)
            }
            Opcode::IntAdd => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();
                let left = self.get_int(input0);
                let right = self.get_int(input1);
                let result = left.add(&right);
                self.write_int(output, &result);
                PcodeControl::Continue
            }
            Opcode::Load => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();

                let offset = input1.offset;
                let space = self.get_space_from_const(input0)?;
                let varnode = VarnodeData { space, offset, size: output.size };

                let bytes = self.get_uint(&varnode);
                self.write_uint(output, &bytes);

                PcodeControl::Continue
            }
            Opcode::Call => {
                let [input0, _args @ ..] = pcode.vars.as_slice() else {
                    bail!("expected at least 1 input");
                };
                PcodeControl::Branch(input0.offset)
            }
            Opcode::IntCarry => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_uint(input0);
                let right = self.get_uint(input1);
                let result = left + right;
                let carry = result.bits() > input0.size as u64;
                self.write_bool(output, carry);
                PcodeControl::Continue
            }
            Opcode::IntSCarry => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_int(input0);
                let right = self.get_int(input1);
                let result = left + right;
                let carry = result.bits() > input0.size as u64;
                self.write_bool(output, carry);
                PcodeControl::Continue
            }
            Opcode::CBranch => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let condition = self.get_uint(input0);

                if condition != BigUint::zero() {
                    PcodeControl::Branch(input1.offset)
                } else {
                    PcodeControl::Continue
                }
            }
            Opcode::Return => {
                let [input0, _values @ ..] = pcode.vars.as_slice() else {
                    bail!("expected at least 1 input");
                };
                let offset = self.get_uint(input0).to_u64()
                    .expect("return value must fit in u64");
                println!("  input0 contained the value: {:0>8X}", offset);
                todo!("interpret return value");
            }
            Opcode::IntXor => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_uint(input0);
                let right = self.get_uint(input1);
                let result = left ^ right;
                self.write_uint(output, &result);
                PcodeControl::Continue
            }
            _ => bail!("unimplemented opcode: {:?}", pcode.opcode),
        };

        Ok(control)
    }

    #[inline]
    fn get_int(&self, varnode: &VarnodeData) -> BigInt {
        if matches!(varnode.space.type_, SpaceType::Constant) {
            println!("  read constant: {:X}", varnode.offset);
            return BigInt::from(varnode.offset);
        }
        let bytes = self.get_bytes(varnode);
        if varnode.space.is_big_endian {
            BigInt::from_signed_bytes_be(bytes.deref())
        } else {
            BigInt::from_signed_bytes_le(bytes.deref())
        }
    }

    #[inline]
    fn get_uint(&self, varnode: &VarnodeData) -> BigUint {
        if matches!(varnode.space.type_, SpaceType::Constant) {
            println!("  read constant: {:X}", varnode.offset);
            return BigUint::from(varnode.offset);
        }
        let bytes = self.get_bytes(varnode);
        if varnode.space.is_big_endian {
            BigUint::from_bytes_be(bytes.deref())
        } else {
            BigUint::from_bytes_le(bytes.deref())
        }
    }

    #[inline]
    fn write_bool(&self, node: &VarnodeData, value: bool) {
        self.write_uint(node, &BigUint::from(value))
    }

    fn get_space_from_const(&self, node: &VarnodeData) -> anyhow::Result<AddrSpace> {
        if node.space.type_ != SpaceType::Constant {
            bail!("expected constant space");
        }

        use sleigh_sys::ffi;
        let space: *mut ffi::AddrSpace = node.offset as *mut ffi::AddrSpace;
        let space: &ffi::AddrSpace = unsafe { // uh oh, now I've done it
            space.as_ref()
                .context("unable to get space")?
        };
        Ok(sleigh::AddrSpace::from(space))
    }
}

struct Slice<'a> {
    bytes: &'a [u8],
    offset: usize,
    size: usize,
}

#[inline]
fn ensure_capacity(dest: &mut Vec<u8>, cap: usize) {
    let start = dest.capacity();
    if cap > start {
        dest.reserve_exact(cap);
        dest.extend(std::iter::repeat(0).take(cap - start));
    }
}

#[inline]
fn to_int_of_size_le(val: &BigInt, out: &mut [u8]) {
    let (sign, bytes) = val.to_bytes_le();
    if bytes.len() > out.len() {
        out.copy_from_slice(&bytes[..out.len()]);
    } else {
        let split = bytes.len();
        out[..split].copy_from_slice(&bytes);
        out[split..].fill(0);
        if matches!(sign, Sign::Minus) {
            for byte in out.iter_mut() {
                *byte = !*byte;
            }
            out[0] = out[0].overflowing_add(1).0;
        }
    }
}

#[inline]
fn to_int_of_size_be(val: &BigInt, out: &mut [u8]) {
    let (sign, bytes) = val.to_bytes_be();
    if val.bits() as usize >= out.len() * 8 {
        let split = bytes.len() - out.len();
        out.copy_from_slice(&bytes[split..]);
    } else {
        let split = out.len() - bytes.len();
        out[split..].copy_from_slice(&bytes);
        out[..split].fill(0);
        if matches!(sign, Sign::Minus) {
            for byte in out.iter_mut() {
                *byte = !*byte;
            }
            let last = out.len() - 1;
            out[last] = out[last].overflowing_add(1).0;
        }
    }
}