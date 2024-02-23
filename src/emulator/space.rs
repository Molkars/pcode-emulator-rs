use std::cell::{RefCell};
use std::collections::BTreeMap;
use num::{BigInt, BigUint, Zero};
use num::bigint::Sign;

#[derive(Default, Debug)]
pub struct Space {
    /// whether the space is big endian or little endian
    #[allow(unused)]
    big_endian: bool,
    /// a map of address to byte
    inner: RefCell<BTreeMap<u64, u8>>,
}

impl Space {
    pub fn new(big_endian: bool) -> Self {
        Self {
            big_endian,
            inner: RefCell::default(),
        }
    }

    pub fn get_bytes(&self, addr: u64, size: u64) -> Vec<u8> {
        let inner = self.inner.borrow_mut();

        let start = addr;
        let end = start + size;
        let mut out = vec![0; size as usize];
        // fill the buffer with the bytes from the map, we manually fill the gaps with 0
        for (key, value) in inner.range(start..end) {
            out[(key - start) as usize] = *value;
        }
        return out;
    }

    pub fn set_bytes(&self, addr: u64, bytes: &[u8]) {
        let mut inner = self.inner.borrow_mut();
        let start = addr;
        for (i, byte) in bytes.iter().enumerate() {
            inner.insert(start + i as u64, *byte);
        }
    }
}

pub trait Read {
    fn read(is_big_endian: bool, src: &[u8]) -> Self;
}

pub trait Write {
    fn write(self, is_big_endian: bool, dest: &mut [u8]);
}

impl Read for BigUint {
    fn read(is_big_endian: bool, out: &[u8]) -> Self {
        if is_big_endian {
            BigUint::from_bytes_be(out)
        } else {
            BigUint::from_bytes_le(out)
        }
    }
}

impl Write for BigUint {
    #[inline]
    fn write(self, is_big_endian: bool, dest: &mut [u8]) {
        Write::write(&self, is_big_endian, dest)
    }
}

impl Write for &'_ BigUint {
    fn write(self, is_big_endian: bool, dest: &mut [u8]) {
        if is_big_endian {
            let be = self.to_bytes_be();
            if be.len() > dest.len() {
                let split = be.len() - dest.len();
                dest.copy_from_slice(&be[split..]);
            } else {
                let split = dest.len() - be.len();
                dest[split..].copy_from_slice(&be);
                dest[..split].fill(0);
            }
        } else {
            let le = self.to_bytes_le();
            if le.len() > dest.len() {
                let split = dest.len();
                dest.copy_from_slice(&le[..split]);
            } else {
                let split = le.len();
                dest[..split].copy_from_slice(&le);
                dest[split..].fill(0);
            }
        }
    }
}

impl Read for BigInt {
    fn read(is_big_endian: bool, src: &[u8]) -> Self {
        if is_big_endian {
            BigInt::from_signed_bytes_be(src)
        } else {
            BigInt::from_signed_bytes_le(src)
        }
    }
}

impl Write for BigInt {
    #[inline]
    fn write(self, is_big_endian: bool, dest: &mut [u8]) {
        Write::write(&self, is_big_endian, dest)
    }
}

impl Write for &'_ BigInt {
    fn write(self, is_big_endian: bool, dest: &mut [u8]) {
        if is_big_endian {
            let (sign, bytes) = self.to_bytes_be();
            if self.bits() as usize >= dest.len() * 8 {
                let split = bytes.len() - dest.len();
                dest.copy_from_slice(&bytes[split..]);
            } else {
                let split = dest.len() - bytes.len();
                dest[split..].copy_from_slice(&bytes);
                dest[..split].fill(0);
                if matches!(sign, Sign::Minus) {
                    for byte in dest.iter_mut() {
                        *byte = !*byte;
                    }
                    let last = dest.len() - 1;
                    dest[last] = dest[last].overflowing_add(1).0;
                }
            }
        } else {
            let (sign, bytes) = self.to_bytes_le();
            if bytes.len() > dest.len() {
                dest.copy_from_slice(&bytes[..dest.len()]);
            } else {
                let split = bytes.len();
                dest[..split].copy_from_slice(&bytes);
                dest[split..].fill(0);
                if matches!(sign, Sign::Minus) {
                    for byte in dest.iter_mut() {
                        *byte = !*byte;
                    }
                    dest[0] = dest[0].overflowing_add(1).0;
                }
            }
        }
    }
}

impl Write for bool {
    fn write(self, is_big_endian: bool, dest: &mut [u8]) {
        let value = BigUint::from(self);
        Write::write(&value, is_big_endian, dest);
    }
}

impl Read for bool {
    fn read(is_big_endian: bool, src: &[u8]) -> Self {
        let value: BigUint = Read::read(is_big_endian, src);
        value != BigUint::zero()
    }
}

macro_rules! primitive {
    ($t:ty; $($n:ty),+) => {
        $(
        impl Read for $n {
            fn read(is_big_endian: bool, src: &[u8]) -> Self {
                <$t as Read>::read(is_big_endian, src)
                .try_into()
                .expect("unable to convert to primitive")
            }
        }

        impl Write for $n {
            fn write(self, is_big_endian: bool, dest: &mut [u8]) {
                <$t>::from(self).write(is_big_endian, dest)
            }
        }
        )+
    };
}

primitive!(BigUint; u8, u16, u32, u64, u128);
primitive!(BigInt; i8, i16, i32, i64, i128);