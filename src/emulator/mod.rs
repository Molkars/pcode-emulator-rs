use std::cell::{Ref, RefCell, RefMut};
use std::mem::MaybeUninit;
use std::ops::{Add, Deref, Index, IndexMut, Sub};
use anyhow::{anyhow, bail};
use hashbrown::HashMap;
use itertools::Itertools;
use num::{BigInt, BigUint, CheckedSub};
use num::bigint::Sign;
use num::traits::{FromBytes, ToBytes};
use sleigh::{AddrSpace, Opcode, PCode, SpaceType, VarnodeData};
use crate::inspect;
use crate::symbol_table::SymbolTable;

struct Space {
    values: Vec<u8>,
}

pub struct Emulator<'a> {
    pcodes: &'a HashMap<String, Vec<PCode>>,
    registers: &'a HashMap<VarnodeData, String>,
    symbol_table: &'a SymbolTable,
    memory: HashMap<AddrSpace, RefCell<Vec<u8>>>,
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

        let mut emulator = Emulator { pcodes, symbol_table, registers, memory: HashMap::new() };

        for register in registers.keys() {
            emulator.memory
                .entry(register.space.clone())
                .or_insert_with(|| RefCell::new(vec![0; 4 << 8]))
                .get_mut()
                .reserve_exact(register.offset as usize + register.size as usize);
        }
        emulator.memory.insert(AddrSpace {
            name: "unique".to_string(),
            type_: SpaceType::Internal,
            is_big_endian: false,
        }, RefCell::new(vec![0; 4 << 8]));

        for pcode in func_pcode {
            println!("{:X} | {:?}", pcode.address, pcode.opcode);
            emulator.emulate_one(pcode)?;
        }

        Ok(())
    }

    #[inline]
    fn get_space(&self, node: &VarnodeData) -> &RefCell<Vec<u8>> {
        let space = self.memory.get(&node.space).unwrap();
        ensure_capacity(space.borrow_mut().as_mut(), node.offset as usize + node.size as usize);
        space
    }

    #[inline]
    fn get_bytes(&self, node: &VarnodeData) -> Ref<[u8]> {
        Ref::map(self.get_space(node).borrow(), |space| {
            &space[node.offset as usize..node.offset as usize + node.size as usize]
        })
    }

    #[inline]
    fn get_bytes_mut(&self, node: &VarnodeData) -> RefMut<[u8]> {
        RefMut::map(self.get_space(node).borrow_mut(), |space| {
            &mut space[node.offset as usize..node.offset as usize + node.size as usize]
        })
    }

    #[inline]
    fn write_int(&self, node: &VarnodeData, value: &BigInt) {
        let mut dest = self.get_bytes_mut(node);
        let value = &BigInt::from(-4);
        const WORD_SIZE: usize = 8;
        if node.space.is_big_endian {
            let (sign, bytes) = value.to_bytes_be();
            let size = node.size as usize;
            if bytes.len() < size * WORD_SIZE {
                let split = size - bytes.len();
                // everything after the split are the big-endian bytes
                dest.index_mut(split..).copy_from_slice(bytes.as_slice());
                // everything before are zeroes
                dest.index_mut(..split).fill(0);
                // if we have a negative, we need to shift that left
                if matches!(sign, Sign::Minus) {
                    dest[0] |= (1 << 7);
                }
            } else {
                dest.copy_from_slice(&bytes[..size]);
            }
        } else {
            let (sign, unsigned) = value.to_bytes_le();

            let node_size = node.size as usize;
            if unsigned.len() >= node_size {
                todo!()
            } else {

            }
        }
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
                if source_node.space == dest_node.space {
                    let mut mem = self.get_space(source_node).borrow_mut();
                    mem.copy_within((source_start..source_start + size), dest_start);
                } else {
                    self.get_bytes_mut(&dest_node).copy_from_slice(self.get_bytes(&source_node).deref());
                }
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
            }
            Opcode::IntSBorrow => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref().unwrap();

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_int(input0);
                let right = self.get_int(input1);
                let overflow = left.checked_sub(&right).is_some();
                let value = BigInt::from(overflow);
                self.write_int(output, &value);
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