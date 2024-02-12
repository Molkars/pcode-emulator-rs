///!
///! This module contains the `Binary` struct and its associated types and functions.
///! The `Binary` struct is used to represent a binary file and its associated metadata.
///! The `Section` struct is used to represent a section within a binary file.
///! The `Symbol` struct is used to represent a symbol within a binary file.
///! The `x86_32` function is used to create a `Binary` struct from a given file path.

use std::collections::{LinkedList};
use std::path::{Path};
use anyhow::{Context};
use hashbrown::HashMap;
use crate::util;

mod readobj;

#[derive(Debug)]
pub struct Binary {
    pub bytes: Vec<u8>,
    pub sections: HashMap<String, Section>,
    pub symbols: HashMap<String, LinkedList<Symbol>>,
}

#[derive(Debug)]
pub struct Section {
    /// the section type
    pub kind: String,
    /// a collection of flags
    pub flags: Vec<String>,
    /// the address of the section
    pub address: u64,
    /// the offset from the start of the file
    pub offset: u64,
    /// the size of the section in bytes
    pub size: u64,
    /// the alignment of the section
    pub alignment: u64,
}

#[derive(Debug)]
pub struct Symbol {
    pub address: u64,
    pub size: u64,
    pub kind: String,
    pub flags: Vec<String>,
    pub section: String,
}

impl Binary {
    pub fn x86_32(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let bytes = util::read_file_as_bytes(path)?;

        let readobj = readobj::read(path)
            .context("unable to run readobj on binary!")?;

        let mut sections = HashMap::new();
        for readobj::SectionItem { section } in readobj.sections {
            assert!(!sections.contains_key(&section.name.name));
            sections.insert(section.name.name, Section {
                kind: section.r#type.name,
                flags: section.flags.flags.into_iter().map(|flag| flag.name).collect(),
                address: section.address,
                offset: section.offset,
                size: section.size,
                alignment: section.address_alignment,
            });
        }

        let mut symbols = HashMap::<_, LinkedList<_>>::new();
        for readobj::SymbolItem { symbol } in readobj.symbols {
            symbols.entry(symbol.name.name)
                .or_default()
                .push_back(Symbol {
                    address: symbol.value,
                    size: symbol.size,
                    kind: symbol.r#type.name,
                    flags: symbol.other.flags.into_iter().map(|flag| flag.name).collect(),
                    section: symbol.section.name,
                });
        }

        Ok(Self {
            bytes,
            sections,
            symbols,
        })
    }
}