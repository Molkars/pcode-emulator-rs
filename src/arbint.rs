use std::fmt::{Debug, Formatter};
use sleigh::Endian;

#[derive(Clone)]
pub struct ArbitraryInt {
    size: u8,
    values: Vec<u8>,
}

#[inline]
fn endian_bytes(bytes: &[u8], endian: Endian) -> Box<dyn Iterator<Item=&u8> + '_> {
    match endian {
        Endian::LittleEndian => Box::new(bytes.iter().rev()),
        Endian::BigEndian => Box::new(bytes.iter()),
    }
}

impl ArbitraryInt {
    pub fn overflowing_add(&self, rhs: Self, endian: Endian) -> (Self, bool) {
        let mut out = ArbitraryInt {
            size: self.size,
            values: vec![0; self.values.len()],
        };

        let lhs_iter = endian_bytes(self.values.as_slice(), endian);
        let rhs_iter = endian_bytes(rhs.values.as_slice(), endian);

        let mut overflow = false;
        for (i, (a, b)) in lhs_iter.zip(rhs_iter).enumerate() {
            let (result, did_overflow) = a.overflowing_add(*b);

            let (result, did_overflow) = if overflow {
                let (result, interior_overflow) = result.overflowing_add(1);
                (result, interior_overflow | did_overflow)
            } else {
                (result, did_overflow)
            };
            overflow = did_overflow;

            let index = match endian {
                Endian::LittleEndian => self.values.len() - i - 1,
                Endian::BigEndian => i,
            };

            let result = if i * 8 > self.size as usize {
                let mask = u8::MAX << (8 - self.size % 8);
                println!("bitset: {:0>8b}", mask);
                result & mask
            } else {
                result
            };

            out.values[index] = result;
        }

        (out, overflow)
    }
}

impl Debug for ArbitraryInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, v) in self.values.iter().enumerate() {
            let length = if (i + 1) * 8 > self.size as usize {
                (i + 1) * 8 - self.size as usize
            } else {
                8
            };
            write!(f, "{:0>length$b}", v, length = length)?;
        }
        Ok(())
    }
}

#[test]
fn test_add() {
    let lhs = ArbitraryInt {
        size: 4,
        values: vec![0b0000_0010],
    };
    let rhs = lhs.clone();

    let (sum, overflowed) = lhs.overflowing_add(rhs.clone(), Endian::LittleEndian);
    println!("overflowed = {overflowed}");
    println!("{lhs:?} + {rhs:?} = {sum:?}");

    let lhs = ArbitraryInt {
        size: 4,
        values: vec![0b0100_0000],
    };
    let rhs = lhs.clone();
    let (sum, overflowed) = lhs.overflowing_add(rhs.clone(), Endian::BigEndian);
    println!("overflow = {overflowed}");
    println!("{lhs:?} + {rhs:?} = {sum:?}");
}