use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeMap;
use std::mem::MaybeUninit;
use std::ops::{Add, Deref, Index, IndexMut, Sub};
use anyhow::{anyhow, bail};
use hashbrown::HashMap;
use itertools::Itertools;
use num::{BigInt, BigUint, CheckedSub, ToPrimitive};
use num::bigint::Sign;
use num::traits::{FromBytes, ToBytes};
use sleigh::{AddrSpace, Opcode, PCode, SpaceType, VarnodeData};
use crate::inspect;
use crate::symbol_table::SymbolTable;

#[derive(Default, Debug)]
struct Space {
    big_endian: bool,
    inner: RefCell<BTreeMap<u64, u8>>,
    buffer: RefCell<Vec<u8>>,
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
    pcodes: &'a HashMap<String, Vec<PCode>>,
    registers: &'a HashMap<VarnodeData, String>,
    symbol_table: &'a SymbolTable,
    memory: HashMap<AddrSpace, Space>,
    ram: Space,
}

impl Emulator<'_> {
    pub fn emulate(
        pcodes: &HashMap<String, Vec<PCode>>,
        func: &str,
        symbol_table: &SymbolTable,
        registers: &HashMap<VarnodeData, String>,
    ) -> anyhow::Result<()> {
        // let func_info = symbol_table.get(func)
        //     .ok_or_else(|| anyhow!("symbol {func:?} not found in symbol-table"))?;

        let func_pcode = pcodes.get(func)
            .ok_or_else(|| anyhow!("symbol {func:?} not found in pcode-table"))?;

        let mut emulator = Emulator {
            pcodes,
            symbol_table,
            registers,
            memory: HashMap::new(),
            ram: Space::new(false),
        };

        for register in registers.keys() {
            emulator.memory
                .entry(register.space.clone())
                .or_insert_with(|| Space::new(register.space.is_big_endian));
        }
        emulator.memory.insert(AddrSpace {
            name: "unique".to_string(),
            type_: SpaceType::Internal,
            is_big_endian: false,
        }, Space::new(false));

        println!("emulating {func} ({} pcodes)", func_pcode.len());
        for pcode in func_pcode {
            println!("{:X} | {:?}", pcode.address, pcode.opcode);
            emulator.emulate_one(pcode)?;
        }

        Ok(())
    }

    // #[inline]
    // fn get_space(&self, node: &VarnodeData) -> &RefCell<Vec<u8>> {
    //     let space = self.memory.get(&node.space).unwrap();
    //     ensure_capacity(space.borrow_mut().as_mut(), node.offset as usize + node.size as usize);
    //     space
    // }

    #[inline]
    fn get_bytes(&self, node: &VarnodeData) -> Ref<[u8]> {
        self.memory.get(&node.space)
            .unwrap_or_else(|| {
                panic!("space not found for node: {:?}", node)
            })
            .get_bytes(node.offset, node.size.into())
    }

    // #[inline]
    // fn get_bytes_mut(&self, node: &VarnodeData) -> RefMut<[u8]> {
    //     RefMut::map(self.get_space(node).borrow_mut(), |space| {
    //         &mut space[node.offset as usize..node.offset as usize + node.size as usize]
    //     })
    // }
    fn set_bytes(&self, node: &VarnodeData, bytes: &[u8]) {
        self.memory.get(&node.space)
            .unwrap()
            .set_bytes(node.offset, bytes);
    }

    #[inline]
    fn write_int(&self, node: &VarnodeData, value: &BigInt) {
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

    fn emulate_one(
        &self,
        pcode: &PCode,
    ) -> anyhow::Result<()> {
        match pcode.opcode {
            Opcode::Copy => {
                let [source_node] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                let dest_node = pcode.outvar.as_ref().unwrap();

                let size = source_node.size as usize;
                let source_start = source_node.offset as usize;
                let dest_start = dest_node.offset as usize;
                let value = self.memory.get(&source_node.space).unwrap()
                    .get_bytes(source_node.offset, size as u64);
                self.memory.get(&dest_node.space).unwrap()
                    .set_bytes(dest_node.offset, value.deref());
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
            }
            Opcode::Store => {
                let [input0, input1, input2] = pcode.vars.as_slice() else {
                    bail!("expected 3 inputs");
                };
                let address = input1.offset;
                let size = input2.size;
                let value = self.ram.get_bytes(input2.offset, size as u64);
                self.ram.set_bytes(address, value.deref());
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
            }
            Opcode::PopCount => {
                let [input0] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                let output = pcode.outvar.as_ref().unwrap();

                let value = self.get_uint(input0);
                let result = value.count_ones();
                self.write_uint(output, &BigUint::from(result));
            }

            _ => bail!("unimplemented opcode: {:?}", pcode.opcode),
        };

        Ok(())
    }

    #[inline]
    fn get_int(&self, varnode: &VarnodeData) -> BigInt {
        let bytes = self.get_bytes(varnode);
        if varnode.space.is_big_endian {
            BigInt::from_signed_bytes_be(bytes.deref())
        } else {
            BigInt::from_signed_bytes_le(bytes.deref())
        }
    }

    #[inline]
    fn get_uint(&self, varnode: &VarnodeData) -> BigUint {
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