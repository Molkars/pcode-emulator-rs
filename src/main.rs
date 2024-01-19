use pcode::*;
use sleigh::Decompiler;

fn main() {
    let mut decompiler = Decompiler::builder().x86(sleigh::X86Mode::Mode32).build();
    let code = b"\x01\xd8";

    let (len, pcodes) = decompiler.translate(code, 0x1000);
    println!("{} {:?}", len, pcodes);

    let (len, insts) = decompiler.disassemble(code, 0x1000);
    println!("{} {:?}", len, insts);
}
