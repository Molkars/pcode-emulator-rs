use cxx::{CxxString, CxxVector};

use num_derive::FromPrimitive;

#[cfg(feature = "serde")]
use serde_derive;

#[derive(Debug, Copy, Clone, FromPrimitive, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde_derive::Serialize))]
pub enum SpaceType {
    Constant = 0,
    Processor = 1,
    SpaceBase = 2,
    Internal = 3,
    Fspec = 4,
    Iop = 5,
    Join = 6,
}

impl SpaceType {
    pub fn from_u32(val: u32) -> Option<Self> {
        num::FromPrimitive::from_u32(val)
    }
}

#[derive(Debug, Copy, Clone, FromPrimitive, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde_derive::Serialize))]
pub enum Opcode {
    ///< Copy one operand to another
    Copy = 1,
    ///< Load from a pointer into a specified address space
    Load = 2,
    ///< Store at a pointer into a specified address space
    Store = 3,
    ///< Always branch
    Branch = 4,
    ///< Conditional branch
    CBranch = 5,
    ///< Indirect branch (jumptable)
    BranchInd = 6,
    ///< Call to an absolute address
    Call = 7,
    ///< Call through an indirect address
    CallInd = 8,
    ///< User-defined operation
    CallOther = 9,
    ///< Return from subroutine
    Return = 10,

    // =====================================================
    // Integer/bit operations
    // =====================================================

    ///< Integer comparison, equality (==)
    IntEqual = 11,
    ///< Integer comparison, in-equality (!=)
    IntNotEqual = 12,
    ///< Integer comparison, signed less-than (<)
    IntSLess = 13,
    ///< Integer comparison, signed less-than-or-equal (<=)
    IntSLessEqual = 14,
    ///< Integer comparison, unsigned less-than (<)
    IntLess = 15,

    // =====================================================
    // This also indicates a borrow on unsigned substraction
    // =====================================================

    ///< Integer comparison, unsigned less-than-or-equal (<=)
    IntLessEqual = 16,
    ///< Zero extension
    IntZExt = 17,
    ///< Sign extension
    IntSExt = 18,
    ///< Addition, signed or unsigned (+)
    IntAdd = 19,
    ///< Subtraction, signed or unsigned (-)
    IntSub = 20,
    ///< Test for unsigned carry
    IntCarry = 21,
    ///< Test for signed carry
    IntSCarry = 22,
    ///< Test for signed borrow
    IntSBorrow = 23,
    ///< Twos complement
    Int2Comp = 24,
    ///< Logical/bitwise negation (~)
    IntNegate = 25,
    ///< Logical/bitwise exclusive-or (^)
    IntXor = 26,
    ///< Logical/bitwise and (&)
    IntAnd = 27,
    ///< Logical/bitwise or (|)
    IntOr = 28,
    ///< Left shift (<<)
    IntLeft = 29,
    ///< Right shift, logical (>>)
    IntRight = 30,
    ///< Right shift, arithmetic (>>)
    IntSRight = 31,
    ///< Integer multiplication, signed and unsigned (*)
    IntMult = 32,
    ///< Integer division, unsigned (/)
    IntDiv = 33,
    ///< Integer division, signed (/)
    IntSDiv = 34,
    ///< Remainder/modulo, unsigned (%)
    IntRem = 35,
    ///< Remainder/modulo, signed (%)
    IntSRem = 36,
    ///< Boolean negate (!)
    BoolNegate = 37,
    ///< Boolean exclusive-or (^^)
    BoolXor = 38,
    ///< Boolean and (&&)
    BoolAnd = 39,
    ///< Boolean or (||)
    BoolOr = 40,

    // =====================================================
    // Floating point operations
    // =====================================================

    ///< Floating-point comparison, equality (==)
    FloatEqual = 41,
    ///< Floating-point comparison, in-equality (!=)
    FloatNotEqual = 42,
    ///< Floating-point comparison, less-than (<)
    FloatLess = 43,
    ///< Floating-point comparison, less-than-or-equal (<=)
    FloatLessEqual = 44,

    // =====================================================
    // Slot 45 is currently unused
    // =====================================================

    ///< Not-a-number test (NaN)
    FloatNan = 46,
    ///< Floating-point addition (+)
    FloatAdd = 47,
    ///< Floating-point division (/)
    FloatDiv = 48,
    ///< Floating-point multiplication (*)
    FloatMult = 49,
    ///< Floating-point subtraction (-)
    FloatSub = 50,
    ///< Floating-point negation (-)
    FloatNeg = 51,
    ///< Floating-point absolute value (abs)
    FloatAbs = 52,
    ///< Floating-point square root (sqrt)
    FloatSqrt = 53,
    ///< Convert an integer to a floating-point
    FloatInt2Float = 54,
    ///< Convert between different floating-point sizes
    FloatFloat2Float = 55,
    ///< Round towards zero
    FloatTrunc = 56,
    ///< Round towards +infinity
    FloatCeil = 57,
    ///< Round towards -infinity
    FloatFloor = 58,
    ///< Round towards nearest
    FloatRound = 59,

    // =====================================================
    // Internal opcodes for simplification. Not
    // typically generated in a direct translation.
    // Data-flow operations
    // =====================================================

    ///< Phi-node operator
    MultiEqual = 60,
    ///< Copy with an indirect effect
    Indirect = 61,
    ///< Concatenate
    Piece = 62,
    ///< Truncate
    SubPiece = 63,
    ///< Cast from one data-type to another
    Cast = 64,
    ///< Index into an array ([])
    PtrAdd = 65,
    ///< Drill down to a sub-field  (->)
    PtrSub = 66,
    ///< Look-up a \e segmented address
    SegmentOp = 67,
    ///< Recover a value from the \e constant \e pool
    CPoolRef = 68,
    ///< Allocate a new object (new)
    New = 69,
    ///< Insert a bit-range
    Insert = 70,
    ///< Extract a bit-range
    Extract = 71,
    ///< Count the 1-bits
    PopCount = 72,
    ///< INT MAX?
    Max = 73,
}

impl Opcode {
    pub fn from_u32(val: u32) -> Option<Self> {
        num::FromPrimitive::from_u32(val)
    }
}

//unsafe impl cxx::ExternType for ffi::spacetype {
//    type Id = type_id!("crate::SpaceType");
//    type Kind = cxx::kind::Trivial;
//}

pub trait AssemblyEmit {
    fn dump(&mut self, addr: &ffi::Address, mnem: &str, body: &str);
}

pub struct RustAssemblyEmit<'a> {
    internal: &'a mut dyn AssemblyEmit,
}

impl<'a> RustAssemblyEmit<'a> {
    pub fn from_internal(internal: &'a mut dyn AssemblyEmit) -> Self {
        Self { internal }
    }

    pub fn dump(&mut self, address: &ffi::Address, mnem: &CxxString, body: &CxxString) {
        let mnem = mnem.to_str().unwrap();
        let body = body.to_str().unwrap();

        self.internal.dump(address, mnem, body);
    }
}

pub trait PCodeEmit {
    /// Callback that will be called when disassembling, emitting the pcode
    /// - address: the address of the machine instruction
    /// - opcode: the opcode of the particular pcode instruction
    /// - outvar: a data about the output varnode
    /// - vars: an array of VarnodeData for each input varnode
    fn dump(
        &mut self,
        address: &ffi::Address,
        opcode: Opcode,
        outvar: Option<&ffi::VarnodeData>,
        vars: &[&ffi::VarnodeData],
    );
}

pub struct RustPCodeEmit<'a> {
    pub internal: &'a mut dyn PCodeEmit,
}

pub trait LoadImage {
    fn load_fill(&mut self, ptr: &mut [u8], addr: &ffi::Address);
    fn adjust_vma(&mut self, _adjust: isize) {}
}

pub struct RustLoadImage<'a> {
    internal: &'a mut dyn LoadImage,
}

impl<'a> RustLoadImage<'a> {
    pub fn from_internal(internal: &'a mut dyn LoadImage) -> Self {
        Self { internal }
    }

    unsafe fn load_fill(&mut self, ptr: *mut u8, size: u32, addr: &ffi::Address) {
        let slice = std::slice::from_raw_parts_mut(ptr, size as usize);
        self.internal.load_fill(slice, addr);
    }

    fn adjust_vma(&mut self, adjust: isize) {
        self.internal.adjust_vma(adjust)
    }
}

impl<'a> RustPCodeEmit<'a> {
    pub fn from_internal(internal: &'a mut dyn PCodeEmit) -> Self {
        Self { internal }
    }

    unsafe fn dump(
        &mut self,
        address: &ffi::Address,
        opcode: u32,
        outvar: *const ffi::VarnodeData,
        vars: &CxxVector<ffi::VarnodeData>,
    ) {
        let outvar = if outvar.is_null() {
            None
        } else {
            Some(&*outvar)
        };
        let vars = vars.iter().collect::<Vec<_>>();
        let opcode = num::FromPrimitive::from_u32(opcode).unwrap();
        self.internal.dump(address, opcode, outvar, vars.as_slice());
    }
}

#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        type RustAssemblyEmit<'a>;
        fn dump(self: &mut RustAssemblyEmit, address: &Address, mnem: &CxxString, body: &CxxString);

        type RustPCodeEmit<'a>;
        unsafe fn dump(
            self: &mut RustPCodeEmit,
            address: &Address,
            opcode: u32,
            outvar: *const VarnodeData,
            vars: &CxxVector<VarnodeData>,
        );

        type RustLoadImage<'a>;
        unsafe fn load_fill(self: &mut RustLoadImage, ptr: *mut u8, size: u32, addr: &Address);
        //fn get_arch_type(self: &RustLoadImage) -> String;
        fn adjust_vma(self: &mut RustLoadImage, adjust: isize);
    }

    unsafe extern "C++" {
        include!("bridge.hh");

        type Address;
        fn isInvalid(self: &Address) -> bool;
        fn getAddrSize(self: &Address) -> i32;
        fn isBigEndian(self: &Address) -> bool;
        fn getSpace(self: &Address) -> *mut AddrSpace;
        fn getOffset(self: &Address) -> u64;
        fn toPhysical(self: Pin<&mut Address>);
        fn getShortcut(self: &Address) -> c_char;
        fn containedBy(self: &Address, sz: i32, op2: &Address, sz2: i32) -> bool;
        fn justifiedContain(
            self: &Address,
            sz: i32,
            op2: &Address,
            sz2: i32,
            forceleft: bool,
        ) -> i32;
        fn overlap(self: &Address, skip: i32, op: &Address, size: i32) -> i32;
        fn isContiguous(self: &Address, sz: i32, loaddr: &Address, losz: i32) -> bool;
        fn isConstant(self: &Address) -> bool;
        fn renormalize(self: Pin<&mut Address>, size: i32);
        fn isJoin(self: &Address) -> bool;

        type VarnodeData;
        fn getVarnodeDataAddress(data: &VarnodeData) -> UniquePtr<Address>;
        fn getVarnodeSpace(data: &VarnodeData) -> *mut AddrSpace;
        fn getVarnodeOffset(data: &VarnodeData) -> u64;
        fn getVarnodeSize(data: &VarnodeData) -> u32;
        fn getVarnode_sizeof() -> u64;

        type spacetype;
        type AddrSpace;
        fn getName(self: &AddrSpace) -> &CxxString;
        //fn getType(self: &AddrSpace) -> spacetype;
        fn getDelay(self: &AddrSpace) -> i32;
        fn getDeadcodeDelay(self: &AddrSpace) -> i32;
        fn getIndex(self: &AddrSpace) -> i32;
        fn getWordSize(self: &AddrSpace) -> u32;
        fn getAddrSize(self: &AddrSpace) -> u32;
        fn getHighest(self: &AddrSpace) -> u64;
        fn getPointerLowerBound(self: &AddrSpace) -> u64;
        fn getPointerUpperBound(self: &AddrSpace) -> u64;
        fn getMinimumPtrSize(self: &AddrSpace) -> i32;
        fn wrapOffset(self: &AddrSpace, off: u64) -> u64;
        fn getShortcut(self: &AddrSpace) -> c_char;
        fn isHeritaged(self: &AddrSpace) -> bool;
        fn doesDeadcode(self: &AddrSpace) -> bool;
        fn hasPhysical(self: &AddrSpace) -> bool;
        fn isBigEndian(self: &AddrSpace) -> bool;
        fn isReverseJustified(self: &AddrSpace) -> bool;
        fn isOverlay(self: &AddrSpace) -> bool;
        fn isOverlayBase(self: &AddrSpace) -> bool;
        fn isOtherSpace(self: &AddrSpace) -> bool;
        fn isTruncated(self: &AddrSpace) -> bool;
        fn hasNearPointers(self: &AddrSpace) -> bool;
        fn numSpacebase(self: &AddrSpace) -> i32;
        fn getSpacebase(self: &AddrSpace, i: i32) -> &VarnodeData;
        fn getSpacebaseFull(self: &AddrSpace, i: i32) -> &VarnodeData;
        fn stackGrowsNegative(self: &AddrSpace) -> bool;
        fn getContain(self: &AddrSpace) -> *mut AddrSpace;

        type OpCode;

        type DocumentStorage;

        type ContextInternal;
        type ContextDatabase;

        fn setVariableDefault(self: Pin<&mut ContextDatabase>, nm: &CxxString, val: u32);
        fn getDefaultValue(self: &ContextDatabase, nm: &CxxString) -> u32;
        fn setVariable(self: Pin<&mut ContextDatabase>, nm: &CxxString, addr: &Address, val: u32);
        fn getVariable(self: &ContextDatabase, nm: &CxxString, addr: &Address) -> u32;

        fn newAddress() -> UniquePtr<Address>;
        fn newContext() -> UniquePtr<ContextDatabase>;
        fn newDocumentStorage(s: &CxxString) -> UniquePtr<DocumentStorage>;

        fn getAddrSpaceType(addr: &AddrSpace) -> u32;

        type Decompiler;
        unsafe fn translate(self: &Decompiler, emit: *mut RustPCodeEmit, addr: u64, limit: u64) -> i32;
        unsafe fn disassemble(self: &Decompiler, emit: *mut RustAssemblyEmit, addr: u64, limit: u64) -> i32;
        unsafe fn getContext(self: Pin<&mut Decompiler>) -> *mut ContextDatabase;
        unsafe fn newDecompiler(
            loadImage: *mut RustLoadImage,
            spec: UniquePtr<DocumentStorage>,
        ) -> UniquePtr<Decompiler>;
        unsafe fn getRegisterList(self: &Decompiler, out: Pin<&mut CxxVector<RegisterPair>>);

        type RegisterPair;
        fn getKey(self: &RegisterPair) -> &CxxString;
        fn getVarnode(self: &RegisterPair) -> &VarnodeData;
    }
}

#[cfg(test)]
mod tests {
    use super::ffi;

    #[test]
    fn test_new() {
        let _a = ffi::newAddress();
        let _a = ffi::newContext();
    }
}
