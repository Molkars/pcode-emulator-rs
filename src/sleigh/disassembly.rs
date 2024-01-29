use cxx::UniquePtr;
use crate::sleigh::DisassemblyInstruction;

pub struct Disassembly(pub(super) UniquePtr<super::ffi::Disassembly>);

impl Disassembly {
    pub fn get_instructions(&self) -> Vec<&DisassemblyInstruction> {
        let inner = self.0.as_ref()
            .expect("unable to get item");
        inner
            .getInstructions()
            .iter()
            .collect()
    }
}