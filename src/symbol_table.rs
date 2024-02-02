use std::borrow::Borrow;
use std::hash::Hash;
use std::path::Path;
use std::process::{Command, exit};
use anyhow::{Context};
use hashbrown::HashMap;

pub struct SymbolTable(HashMap<String, SymbolInfo>);

#[derive(Debug)]
pub struct SymbolInfo {
    pub address: u64,
    pub section: String,
    pub size: u64,
    pub flags: String,
}


impl SymbolTable {
    #[inline]
    pub fn get<Q: ?Sized + Hash + Eq>(&self, key: &Q) -> Option<&SymbolInfo>
        where String: Borrow<Q>
    {
        self.0.get(key)
    }

    pub fn iter(&self) -> impl Iterator<Item=(&str, &SymbolInfo)> + '_ {
        self.0.iter()
            .map(|(k, v)| (k.as_str(), v))
    }
}

impl SymbolTable {
    pub fn build_symbol_table(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let symbol_table = Command::new("llvm-objdump")
            .arg("-t")
            .arg(path.as_ref())
            .output();
        let symbol_table = match symbol_table {
            Ok(output) => {
                String::from_utf8(output.stdout)
                    .unwrap_or_else(|_| {
                        eprintln!("unable to parse objdump output");
                        exit(1);
                    })
            }
            Err(e) => {
                eprintln!("unable to run objdump on the specified binary: {}", e);
                exit(1);
            }
        };
        symbol_table
            .lines()
            .skip(4)
            .filter_map(|line| {
                (|| {
                    let (address, rest) = line.split_once(' ')
                        .context("unable to splice base address")?;
                    let address = u64::from_str_radix(address, 16)
                        .context("unable to parse base address")?;

                    let flags = rest[..7].to_string();
                    let rest = &rest[8..];
                    let (section, rest) = rest.split_once('\t')
                        .context("unable to slice section")?;
                    let (size, name) = rest.split_once(' ')
                        .context("unable to slice entry size")?;
                    let size = u64::from_str_radix(size, 16)
                        .context("unable to parse entry size")?;

                    if !flags.contains('F') {
                        return Ok(None);
                    }

                    Ok(Some((name.to_string(), SymbolInfo {
                        address,
                        section: section.to_string(),
                        size,
                        flags: flags.to_string(),
                    })))
                })().transpose()
            })
            .collect::<anyhow::Result<hashbrown::HashMap<_, _>>>()
            .map(Self)
    }
}