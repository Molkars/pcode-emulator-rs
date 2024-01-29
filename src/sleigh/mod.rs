#![allow(unused)]

use std::ffi::c_char;

mod sleigh_bridge;
mod address;
mod disassembly;
mod disassembly_instruction;

pub use sleigh_bridge::*;
pub use address::*;
pub use disassembly::*;
pub use disassembly_instruction::*;

#[repr(C)]
pub struct uintb(u8);

unsafe impl cxx::ExternType for uintb {
    type Id = cxx::type_id!("uintb");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    #[allow(non_camel_case_types)]
    unsafe extern "C++" {
        include!("pcode/include/bridge.hh");

        type uintb = crate::sleigh::uintb;

        type Address;
        fn isInvalid(self: &Address) -> bool;
        fn getAddrSize(self: &Address) -> i32;
        fn isBigEndian(self: &Address) -> bool;
        fn getSpace(self: &Address) -> *mut AddrSpace;
        fn getOffset(self: &Address) -> uintb;

        type AddrSpace;

        type SleighBridge;
        pub fn create_sleigh_bridge(path: &CxxString) -> UniquePtr<SleighBridge>;
        // fn disassemble(self: Pin<&mut SleighBridge>, bytes: &[u8], len: u32, addr: u64, max_instructions: u32) -> UniquePtr<Disassembly>;
        fn SleighBridge_disassemble(bridge: Pin<&mut SleighBridge>, bytes: &[u8], len: u32, addr: u64, max_instructions: u32) -> UniquePtr<Disassembly>;

        type Disassembly;
        pub fn getInstructions(self: &Disassembly) -> &CxxVector<DisassemblyInstruction>;

        type DisassemblyInstruction;
        pub fn getAddress(self: Pin<&DisassemblyInstruction>) -> &Address;
        pub fn getLength(self: Pin<&DisassemblyInstruction>) -> u64;
        pub fn getMNEM(self: Pin<&DisassemblyInstruction>) -> &CxxString;
        pub fn getBody(self: Pin<&DisassemblyInstruction>) -> &CxxString;
    }
}