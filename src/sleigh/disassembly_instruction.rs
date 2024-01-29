use std::pin::{Pin};
use crate::sleigh::{Address, ffi};

pub struct DisassemblyInstruction(Pin<Box<ffi::DisassemblyInstruction>>);

impl DisassemblyInstruction {
    pub fn len(&self) -> usize {
        usize::try_from(self.0.as_ref().getLength()).unwrap()
    }

    pub fn mnem(&self) -> String {
        self.0.as_ref().getMNEM().to_string()
    }

    pub fn body(&self) -> String {
        self.0.as_ref().getBody().to_string()
    }

    pub fn address(&self) -> Address {
        Address::from(self.0.as_ref().getAddress())
    }
}