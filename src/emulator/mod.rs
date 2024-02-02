use anyhow::{anyhow, bail};
use hashbrown::HashMap;
use sleigh::PCode;
use crate::symbol_table::SymbolTable;

pub struct Emulator {}

impl Emulator {
    pub fn emulate(pcodes: &HashMap<String, Vec<PCode>>, func: &str, symbol_table: &SymbolTable)
        -> anyhow::Result<()>
    {
        // let func_info = symbol_table.get(func)
        //     .ok_or_else(|| anyhow!("symbol {func:?} not found in symbol-table"))?;

        let pcodes = pcodes.get(func)
            .ok_or_else(|| anyhow!("symbol {func:?} not found in pcode-table"))?;

        let mut emulator = Emulator {};
        for pcode in pcodes {
            println!("{} | {:?}", pcode.address, pcode.opcode);
            emulator.emulate_one(pcode)?;
        }

        Ok(())
    }

    fn emulate_one(
        &mut self,
        pcode: &PCode
    ) -> anyhow::Result<()> {
        match pcode.opcode {

            _ => bail!("unimplemented opcode: {:?}", pcode.opcode),
        };

        Ok(())
    }
}