use std::ops::Index;
use anyhow::{anyhow, bail};
use sleigh::{Opcode, PCode, VarnodeData};
use hashbrown::HashMap;
use num::BigInt;
use num::traits::{FromBytes, ToBytes};

struct Space(Vec<u8>);

impl Space {
    #[inline]
    fn with_size(size: usize) -> Self {
        Self(vec![0; size])
    }
}

struct Val {}

struct Machine {
    spaces: HashMap<String, Space>,
}

impl Machine {
    fn get_value(&self, node: &VarnodeData) -> Option<&[u8]> {
        let space = self.spaces.get(node.space.name.as_str())?;
        let off = node.offset as usize;
        Some(space.0.index(off..off + node.size as usize))
    }

    fn set_value(&mut self, node: &VarnodeData, value: &BigInt) -> Option<()> {
        let size = value.bits().div_ceil(8);
        assert_eq!(node.size as u64, size);
        let space = self.spaces.get_mut(node.space.name.as_str())?;
        space.0.copy_from_slice(value.to_le_bytes().as_slice());
        Some(())
    }
}

pub struct Emulator;

impl Emulator {
    pub fn emulate<'a>(codes: impl IntoIterator<Item=&'a PCode>) -> anyhow::Result<()> {
        let machine = Machine {
            spaces: HashMap::from([
                ("register".to_string(), Space::with_size(1024)),
                ("unique".to_string(), Space::with_size(u16::MAX as usize)),
            ]),
        };

        for code in codes.into_iter() {
            println!("{:?} --> {:?}", code.vars, code.outvar);
            match code.opcode {
                Opcode::IntSBorrow => {
                    let [lhs, rhs] = code.vars.as_slice() else {
                        bail!("IntSBorrow: requires two input values");
                    };

                    if lhs.size != rhs.size {
                        bail!("IntSBorrow: input values must have identical sizes");
                    }

                    let lhs_value = machine.get_value(lhs)
                        .ok_or_else(|| anyhow!("lhs did not resolve to a value"))?;
                    let rhs_value = machine.get_value(rhs)
                        .ok_or_else(|| anyhow!("rhs did not resolve to a value"))?;
                    let lhs_value = BigInt::from_le_bytes(lhs_value);
                    let rhs_value = BigInt::from_le_bytes(rhs_value);
                    println!("values: int_sborrow {lhs_value} {rhs_value}");
                    let overflow = if lhs_value.sign() == rhs_value.sign() {
                        false
                    } else {
                        lhs_value.checked_sub(&rhs_value).is_some()
                    };
                }
                _ => return Err(anyhow!("unimplemented: {:?}", code.opcode)),
            };
        }

        Ok(())
    }
}

fn integer_add(lhs: &[u8], rhs: &[u8]) -> (Vec<u8>, bool) {
    assert_eq!(lhs.len(), rhs.len());
    let mut out = vec![0; lhs.len()];
    let mut overflow = false;
    for (i, (a, b)) in lhs.iter().rev().zip(rhs.iter().rev()).enumerate() {
        let (result, did_overflow) = a.overflowing_add(*b);

        let (result, did_overflow) = if overflow {
            let (result, interior_overflow) = result.overflowing_add(1);
            (result, interior_overflow | did_overflow)
        } else {
            (result, did_overflow)
        };
        overflow = did_overflow;
        out[lhs.len() - i - 1] = result;
    }

    (out, overflow)
}

#[inline]
fn negate_integer(value: &[u8]) -> Vec<u8> {
    value
        .iter()
        .map(|val| !*val)
        .collect()
}

fn ssub(lhs: &[u8], rhs: &[u8]) -> (Vec<u8>, bool) {
    let complement = negate_integer(rhs);
    let (result, overflow) = integer_add(lhs, complement.as_slice());
    let (result, overflow2) = integer_add(result.as_slice(), &[1]);
    (result, overflow || overflow2)
}