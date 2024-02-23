use std::hash::Hash;
use std::ops::Deref;
use anyhow::{bail, Context};
use hashbrown::{Equivalent, HashMap};
use itertools::Itertools;
use num::{BigInt, BigUint, Zero};
use sleigh::{AddrSpace, Opcode, PCode, SpaceType, VarnodeData};
use crate::emulator::{Space, space};

macro_rules! log {
    () => {
        println!();
    };
    ($($t:tt)+) => {
        println!($($t)+);
    };
}

/// A control structure for the emulator
pub enum PCodeControl {
    /// branch to the given address
    Branch(u64),
    /// continue to the next pcode instruction
    Continue,
}

pub struct Emulator {
    pub unique_space: Space,
    pub register_space: Space,
    pub ram_space: Space,
    pub register_names: HashMap<VarnodeData, String>,
    pub named_registers: HashMap<String, VarnodeData>,
}

impl Emulator {
    pub fn new(register_names: HashMap<VarnodeData, String>) -> Self {
        let named_registers = register_names
            .iter()
            .map(|(k, v)| (v.clone(), k.clone()))
            .collect();
        Self {
            unique_space: Space::new(false),
            register_space: Space::new(false),
            ram_space: Space::new(false),
            register_names,
            named_registers,
        }
    }

    #[inline]
    pub fn get_register<Q: ?Sized>(&self, k: &Q) -> Option<&VarnodeData>
        where
            Q: Hash + Equivalent<String>,
    {
        self.named_registers.get(k)
    }

    #[inline]
    pub fn get_register_name<Q: ?Sized>(&self, k: &Q) -> Option<&String>
        where
            Q: Hash + Equivalent<VarnodeData>,
    {
        self.register_names.get(k)
    }
}

impl Emulator {
    #[inline]
    pub fn get_bytes(&self, node: &VarnodeData) -> Vec<u8> {
        let bytes = self.get_varnode_space(node)
            .expect("sleigh dropped the ball, node doesn't have a space")
            .get_bytes(node.offset, node.size.into());
        log!("  read {:X?} from {}", bytes, self.nameof(node));
        bytes
    }

    pub fn set_bytes(&self, node: &VarnodeData, bytes: &[u8]) {
        if matches!(node.space.type_, SpaceType::Constant) {
            panic!("cannot write to constant space");
        }
        log!("  wrote {:X?} to {}", bytes, self.nameof(node));
        self.get_varnode_space(node)
            .unwrap()
            .set_bytes(node.offset, bytes);
    }

    pub fn read<T: space::Read>(&self, node: &VarnodeData) -> T {
        if matches!(node.space.type_, SpaceType::Constant) {
            let bytes = if node.space.is_big_endian {
                node.offset.to_be_bytes()
            } else {
                node.offset.to_le_bytes()
            };
            T::read(node.space.is_big_endian, &bytes)
        } else {
            T::read(node.space.is_big_endian, self.get_bytes(node).deref())
        }
    }

    #[inline]
    pub fn write<T: space::Write>(&self, node: &VarnodeData, value: T) {
        let mut bytes = vec![0; node.size as usize];
        value.write(node.space.is_big_endian, &mut bytes);
        self.set_bytes(node, &bytes);
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
        let out = sleigh::AddrSpace::from(space);
        log!("  resolved {} to the {:?} space", self.nameof(node), out.name);
        Ok(out)
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
        self.register_names.get(node)
            .cloned()
            .unwrap_or_else(|| format!("{}:{:X}+{}", node.space.name, node.offset, node.size))
    }

    pub fn emulate_one(
        &self,
        pcode: &PCode,
    ) -> anyhow::Result<PCodeControl> {
        log!("  {:?} : {} -> {}", pcode.opcode,
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
                let value: BigUint = self.read(input0);
                self.write(output, value);

                PCodeControl::Continue
            }
            Opcode::IntSub => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left: BigInt = self.read(input0);
                let right: BigInt = self.read(input1);
                let result = left - right;
                self.write(output, result);
                PCodeControl::Continue
            }
            Opcode::Store => {
                let [input0, input1, input2] = pcode.vars.as_slice() else {
                    bail!("expected 3 inputs");
                };

                let space = self.get_space_from_const(input0)?;
                let offset: u64 = self.read(input1);
                let offset = offset * u64::from(space.wordsize); // offset to bytes
                let value: BigUint = self.read(input2);

                let varnode = VarnodeData { space, offset, size: input2.size };
                self.write(&varnode, value);

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

                let left: BigInt = self.read(input0);
                let right: BigInt = self.read(input1);
                let result = left - right;
                let overflow = result.bits() > u64::from(input0.size);
                self.write(output, overflow);
                PCodeControl::Continue
            }
            Opcode::IntLess => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");

                let left: BigUint = self.read(input0);
                let right: BigUint = self.read(input1);
                self.write(output, left < right);
                PCodeControl::Continue
            }
            Opcode::IntSLess => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left: BigInt = self.read(input0);
                let right: BigInt = self.read(input1);
                self.write(output, left < right);
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

                let left: BigUint = self.read(input0);
                let right: BigUint = self.read(input1);
                let result = left == right;
                self.write(output, result);
                PCodeControl::Continue
            }
            Opcode::IntAnd => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left: BigUint = self.read(input0);
                let right: BigUint = self.read(input1);
                let result = &left & &right;
                self.write(output, result);
                PCodeControl::Continue
            }
            Opcode::PopCount => {
                let [input0] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let value: BigUint = self.read(input0);
                let result = value.count_ones();
                self.write(output, result);
                PCodeControl::Continue
            }
            Opcode::Branch => {
                let [addr] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                log!("  branch to {:X}", addr.offset);
                PCodeControl::Branch(addr.offset)
            }
            Opcode::IntAdd => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let left: BigInt = self.read(input0);
                let right: BigInt = self.read(input1);
                let result = &left + &right;
                self.write(output, result);
                PCodeControl::Continue
            }
            Opcode::Load => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let space = self.get_space_from_const(input0)?;
                let offset: u64 = self.read(input1);
                let offset = offset * u64::from(space.wordsize); // offset to bytes
                let varnode = VarnodeData { space, offset, size: output.size };

                let bytes: BigUint = self.read(&varnode);
                self.write(output, bytes);

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

                let left: BigUint = self.read(input0);
                let right: BigUint = self.read(input1);
                let result = &left + &right;
                let carry = result.bits() > input0.size as u64;
                self.write(output, carry);
                PCodeControl::Continue
            }
            Opcode::IntSCarry => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");

                let left: BigInt = self.read(input0);
                let right: BigInt = self.read(input1);
                let result = &left + &right;
                let carry = result.bits() > input0.size as u64;
                self.write(output, carry);
                PCodeControl::Continue
            }
            Opcode::CBranch => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };

                let condition: BigUint = self.read(input1);
                if condition != BigUint::zero() {
                    log!("  branch to {:X}", input0.offset);
                    PCodeControl::Branch(input0.offset)
                } else {
                    log!("  fall through");
                    PCodeControl::Continue
                }
            }
            Opcode::Return => {
                let [input0, _values @ ..] = pcode.vars.as_slice() else {
                    bail!("expected at least 1 input");
                };
                let off: u64 = self.read(input0);
                log!("  return to {:X}", off);
                PCodeControl::Branch(off)
            }
            Opcode::IntXor => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left: BigUint = self.read(input0);
                let right: BigUint = self.read(input1);
                let result = &left ^ &right;
                self.write(output, result);
                PCodeControl::Continue
            }
            Opcode::IntOr => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                assert_eq!(input0.size, input1.size, "inputs must have be the same size");
                let left: BigUint = self.read(input0);
                let right: BigUint = self.read(input1);
                let result = &left | &right;
                self.write(output, result);
                PCodeControl::Continue
            }
            Opcode::IntZExt => {
                let [input0] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let value: BigUint = self.read(input0);
                self.write(output, value);
                PCodeControl::Continue
            }
            Opcode::BoolOr => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let left: bool = self.read(input0);
                let right: bool = self.read(input1);
                let result = left | right;
                self.write(output, result);
                PCodeControl::Continue
            }
            Opcode::BoolXor => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let left: bool = self.read(input0);
                let right: bool = self.read(input1);
                let result = left ^ right;
                self.write(output, result);
                PCodeControl::Continue
            }
            Opcode::BoolNegate => {
                let [input0] = pcode.vars.as_slice() else {
                    bail!("expected 1 input");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let value: bool = self.read(input0);
                let result = !&value;
                self.write(output, result);
                PCodeControl::Continue
            }
            Opcode::BoolAnd => {
                let [input0, input1] = pcode.vars.as_slice() else {
                    bail!("expected 2 inputs");
                };
                let output = pcode.outvar.as_ref()
                    .context("expected output")?;

                let left: bool = self.read(input0);
                let right: bool = self.read(input1);
                let result = left & right;
                self.write(output, result);
                PCodeControl::Continue
            }
            _ => bail!("unimplemented opcode: {:?}", pcode.opcode),
        };

        Ok(control)
    }
}
