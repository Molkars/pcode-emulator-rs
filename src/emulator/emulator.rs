use std::cell::Ref;
use std::collections::btree_map;
use std::hash::Hash;
use std::ops::Deref;
use anyhow::{bail, Context};
use hashbrown::Equivalent;
use itertools::Itertools;
use num::{BigInt, BigUint, ToPrimitive, Zero};
use num::bigint::Sign;
use sleigh::{AddrSpace, Opcode, PCode, SpaceType, VarnodeData};
use crate::emulator::{Machine, Space};


/// A control structure for the emulator
pub enum PCodeControl {
    /// branch to the given address
    Branch(u64),
    /// continue to the next pcode instruction
    Continue,
}

// todo: actually use this for generic read/write functions
pub trait IntoBytes {
    fn into_bytes(self, is_big_endian: bool, out: &mut [u8]);
}

pub struct Emulator<'a, 'b> {
    /// the emulator
    pub emulator: &'a Machine<'b>,
    /// the current instruction address
    pub address: u64,
    /// the exit address of the emulator code
    pub end_address: u64,

    pub unique_space: Space,
    pub register_space: Space,
    pub ram_space: Space,

    pcode_group_iter: btree_map::Range<'a, u64, Vec<PCode>>,
    pcode_iter: std::iter::Enumerate<std::slice::Iter<'a, PCode>>,
}

impl<'a, 'b> Emulator<'a, 'b> {
    pub fn new(machine: &'a Machine<'b>, address: u64, end_address: u64) -> Self {
        let mut pcode_group_iter = machine.pcodes.range(address..);
        let (new_addr, new_vec) = pcode_group_iter.next().expect("no more pcodes!");
        let pcode_iter = new_vec.iter().enumerate();
        Self {
            emulator: machine,
            address: *new_addr,
            end_address,
            unique_space: Space::new(false),
            register_space: Space::new(false),
            ram_space: Space::new(false),
            pcode_group_iter,
            pcode_iter,
        }
    }

    #[inline]
    pub fn set_address(&mut self, address: u64) {
        if self.address == self.end_address {
            return;
        }
        self.pcode_group_iter = self.emulator.pcodes.range(address..);
        let (new_addr, new_vec) = self.pcode_group_iter.next().expect("no more pcodes!");
        self.address = *new_addr;
        self.pcode_iter = new_vec.iter().enumerate();
    }

    #[inline]
    pub fn get_register<Q: ?Sized>(&self, k: &Q) -> Option<&VarnodeData>
        where
            Q: Hash + Equivalent<String>,
    {
        self.emulator.named_registers.get(k)
    }
}

impl<'a, 'b> Iterator for Emulator<'a, 'b> {
    type Item = (usize, &'a PCode);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let Some((i, pcode)) = self.pcode_iter.next() else {
            if self.address == self.end_address {
                return None;
            }
            let (new_addr, new_vec) = self.pcode_group_iter.next().expect("no more pcodes!");
            self.address = *new_addr;
            self.pcode_iter = new_vec.iter().enumerate();
            return self.next();
        };
        Some((i, pcode))
    }
}

impl<'a, 'b> Emulator<'a, 'b> {
    #[inline]
    pub fn get_bytes(&self, node: &VarnodeData) -> Ref<[u8]> {
        let bytes = self.get_varnode_space(node)
            .expect("sleigh dropped the ball, node doesn't have a space")
            .get_bytes(node.offset, node.size.into());
        println!("  read {:X?} from {}", bytes, self.nameof(node));
        bytes
    }

    pub fn set_bytes(&self, node: &VarnodeData, bytes: &[u8]) {
        if matches!(node.space.type_, SpaceType::Constant) {
            panic!("cannot write to constant space");
        }
        println!("  wrote {:X?} to {}", bytes, self.nameof(node));
        self.get_varnode_space(node)
            .unwrap()
            .set_bytes(node.offset, bytes);
    }

    #[inline]
    pub fn write_int(&self, node: &VarnodeData, value: &BigInt) {
        println!("  wrote {:X} to {}", value, self.nameof(node));
        let mut bytes = vec![0; node.size as usize];
        if node.space.is_big_endian {
            to_int_of_size_be(value, bytes.as_mut());
        } else {
            to_int_of_size_le(value, bytes.as_mut());
        }
        self.set_bytes(node, &bytes);
    }

    #[inline]
    pub fn get_int(&self, varnode: &VarnodeData) -> BigInt {
        if matches!(varnode.space.type_, SpaceType::Constant) {
            println!("  read constant: {:X}", varnode.offset);
            return BigInt::from(varnode.offset);
        }
        let bytes = self.get_bytes(varnode);
        let value = if varnode.space.is_big_endian {
            BigInt::from_signed_bytes_be(bytes.deref())
        } else {
            BigInt::from_signed_bytes_le(bytes.deref())
        };
        println!("  read {:X} from {}", value, self.nameof(varnode));
        value
    }

    #[inline]
    pub fn write_uint(&self, node: &VarnodeData, value: &BigUint) {
        println!("  wrote {:X} to {}", value, self.nameof(node));
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
    pub fn get_uint(&self, varnode: &VarnodeData) -> BigUint {
        if matches!(varnode.space.type_, SpaceType::Constant) {
            println!("  read constant: {:X}", varnode.offset);
            return BigUint::from(varnode.offset);
        }
        let bytes = self.get_bytes(varnode);
        let value = if varnode.space.is_big_endian {
            BigUint::from_bytes_be(bytes.deref())
        } else {
            BigUint::from_bytes_le(bytes.deref())
        };
        println!("  read {:X} from {}", value, self.nameof(varnode));
        value
    }

    #[inline]
    pub fn write_bool(&self, node: &VarnodeData, value: bool) {
        self.write_uint(node, &BigUint::from(value))
    }

    #[inline]
    pub fn get_bool(&self, node: &VarnodeData) -> bool {
        !self.get_uint(node).is_zero()
    }

    pub fn get_space_from_const(&self, node: &VarnodeData) -> anyhow::Result<AddrSpace> {
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

    #[inline]
    pub fn get_varnode_space(&self, node: &VarnodeData) -> anyhow::Result<&Space> {
        self.get_space(&node.space.name)
    }

    #[inline]
    pub fn get_space(&self, name: &str) -> anyhow::Result<&Space> {
        match name {
            "unique" => Ok(&self.unique_space),
            "register" => Ok(&self.register_space),
            "ram" => Ok(&self.ram_space),
            _ => bail!("unsupported space type: {:?}", name),
        }
    }

    pub fn nameof(&self, node: &VarnodeData) -> String {
        self.emulator.register_names.get(node)
            .cloned()
            .unwrap_or_else(|| format!("{}:{:X}+{}", node.space.name, node.offset, node.size))
    }

    pub fn emulate_one(
        &self,
        pcode: &PCode,
    ) -> anyhow::Result<PCodeControl> {
        println!("  {:?} : {} -> {}", pcode.opcode,
                 pcode.vars.iter().map(|node| self.nameof(node)).join(", "),
                 pcode.outvar.as_ref().map(|node| self.nameof(node)).unwrap_or("!".to_string()));
        let control = match pcode.opcode {
            Opcode::Copy => {
                let [input0] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, output.size, "input and output must have the same size");
                let value = self.get_uint(input0);
                self.write_uint(output, &value);

                PCodeControl::Continue
            }
            Opcode::IntSub => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_int(input0);
                let right = self.get_int(input1);
                let result = left - right;
                self.write_int(output, &result);
                PCodeControl::Continue
            }
            Opcode::Store => {
                let [input0, input1, input2] = pcode.vars.as_slice() else {
                    bail!("expected 3 inputs");
                };

                let space = self.get_space_from_const(input0)?;
                let offset = self.get_uint(input1).to_u64()
                    .context("offset must fit in u64")?;
                let offset = offset * u64::from(space.wordsize); // offset to bytes
                let value = self.get_uint(input2);

                let varnode = VarnodeData { space, offset, size: input2.size };
                self.write_uint(&varnode, &value);

                PCodeControl::Continue
            }
            Opcode::IntSBorrow => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                assert_eq!(output.size, 1, "output must be 1 byte");

                let left = self.get_int(input0);
                let right = self.get_int(input1);
                let result = &left - &right;
                let overflow = result.bits() > u64::from(input0.size);
                self.write_bool(output, overflow);
                PCodeControl::Continue
            }
            Opcode::IntLess => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");

                let left = self.get_uint(input0);
                let right = self.get_uint(input1);
                self.write_bool(output, left < right);
                PCodeControl::Continue
            }
            Opcode::IntSLess => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_int(input0);
                let right = self.get_int(input1);
                self.write_bool(output, left < right);
                PCodeControl::Continue
            }
            Opcode::IntEqual => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                assert_eq!(output.size, 1, "output must be 1 byte");

                let left = self.get_uint(input0);
                let right = self.get_uint(input1);
                let result = left == right;
                self.write_bool(output, result);
                PCodeControl::Continue
            }
            Opcode::IntAnd => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_uint(input0);
                let right = self.get_uint(input1);
                let result = &left & &right;
                self.write_uint(output, &result);
                PCodeControl::Continue
            }
            Opcode::PopCount => {
                let [input0] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let value = self.get_uint(input0);
                let result = value.count_ones();
                self.write_uint(output, &BigUint::from(result));
                PCodeControl::Continue
            }
            Opcode::Branch => {
                let [addr] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                println!("  branch to {:X}", addr.offset);
                PCodeControl::Branch(addr.offset)
            }
            Opcode::IntAdd => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let left = self.get_int(input0);
                let right = self.get_int(input1);
                let result = &left + &right;
                self.write_int(output, &result);
                PCodeControl::Continue
            }
            Opcode::Load => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let space = self.get_space_from_const(input0)?;
                let offset = self.get_uint(input1).to_u64()
                    .context("offset must fit in u64")?;
                let offset = offset * u64::from(space.wordsize); // offset to bytes
                let varnode = VarnodeData { space, offset, size: output.size };

                let bytes = self.get_uint(&varnode);
                self.write_uint(output, &bytes);

                PCodeControl::Continue
            }
            Opcode::Call => {
                let [input0, _args @ ..] = pcode.vars.as_slice() else {
                    bail!("expected at least 1 input");
                };
                PCodeControl::Branch(input0.offset)
            }
            Opcode::IntCarry => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");

                let left = self.get_uint(input0);
                let right = self.get_uint(input1);
                let result = &left + &right;
                let carry = result.bits() > input0.size as u64;
                self.write_bool(output, carry);
                PCodeControl::Continue
            }
            Opcode::IntSCarry => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");

                let left = self.get_int(input0);
                let right = self.get_int(input1);
                let result = &left + &right;
                let carry = result.bits() > input0.size as u64;
                self.write_bool(output, carry);
                PCodeControl::Continue
            }
            Opcode::CBranch => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };

                let condition = self.get_uint(input1);
                if condition != BigUint::zero() {
                    println!("  branch to {:X}", input0.offset);
                    PCodeControl::Branch(input0.offset)
                } else {
                    println!("  fall through");
                    PCodeControl::Continue
                }
            }
            Opcode::Return => {
                let [input0, _values @ ..] = pcode.vars.as_slice() else {
                    bail!("expected at least 1 input");
                };
                let off = self.get_uint(input0)
                    .to_u64()
                    .context("offset must fit in u64")?;
                println!("  return to {:X}", off);
                PCodeControl::Branch(off)
            }
            Opcode::IntXor => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_uint(input0);
                let right = self.get_uint(input1);
                let result = &left ^ &right;
                self.write_uint(output, &result);
                PCodeControl::Continue
            }
            Opcode::IntOr => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left = self.get_uint(input0);
                let right = self.get_uint(input1);
                let result = &left | &right;
                self.write_uint(output, &result);
                PCodeControl::Continue
            }
            Opcode::IntZExt => {
                let [input0] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let value = self.get_uint(input0);
                self.write_uint(output, &value);
                PCodeControl::Continue
            }
            Opcode::BoolOr => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let left = self.get_bool(input0);
                let right = self.get_bool(input1);
                let result = left | right;
                self.write_bool(output, result);
                PCodeControl::Continue
            }
            Opcode::BoolXor => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let left = self.get_bool(input0);
                let right = self.get_bool(input1);
                let result = left ^ right;
                self.write_bool(output, result);
                PCodeControl::Continue
            }
            Opcode::BoolNegate => {
                let [input0] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let value = self.get_bool(input0);
                let result = !&value;
                self.write_bool(output, result);
                PCodeControl::Continue
            }
            Opcode::BoolAnd => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let left = self.get_bool(input0);
                let right = self.get_bool(input1);
                let result = left & right;
                self.write_bool(output, result);
                PCodeControl::Continue
            }
            _ => bail!("unimplemented opcode: {:?}", pcode.opcode),
        };

        Ok(control)
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
