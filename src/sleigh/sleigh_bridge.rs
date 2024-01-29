use std::pin::{pin, Pin};
use cxx::{CxxString, UniquePtr};
use crate::sleigh::ffi::{Disassembly, SleighBridge_disassemble};

pub struct SleighBridge {
    inner: UniquePtr<super::ffi::SleighBridge>,
}

impl SleighBridge {
    pub fn new(path: impl AsRef<[u8]>) -> Self {
        cxx::let_cxx_string!(cxx_path = path);
        Self {
            inner: super::ffi::create_sleigh_bridge(&cxx_path),
        }
    }
    
    pub fn disassemble(&mut self, bytes: &[u8], addr: u64) -> super::disassembly::Disassembly {
        let inner = self.inner.as_mut()
            .expect("unable to get pointer");
        let len: u32 = bytes.len() as u32;
        let inner = SleighBridge_disassemble(inner, bytes, len, addr, 0);
        super::disassembly::Disassembly(inner)
    }
}