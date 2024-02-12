use std::path::Path;
use anyhow::Context;
use serde_derive::Deserialize;
use crate::util::{exec, ExecUtil};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct FileSummary {
    pub file: String,
    pub format: String,
    pub arch: String,
    pub address_size: String,
    pub load_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Section {
    pub name: NameValue,
    pub r#type: NameValue,
    pub flags: Flags,
    pub address: u64,
    pub offset: u64,
    pub size: u64,
    pub address_alignment: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Flags {
    pub flags: Vec<NameValue>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Symbol {
    pub name: NameValue,
    pub value: u64,
    pub size: u64,
    pub binding: NameValue,
    pub r#type: NameValue,
    pub section: NameValue,
    pub other: Flags,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SectionItem {
    pub section: Section,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SymbolItem {
    pub symbol: Symbol,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct NameValue {
    pub name: String,
    pub value: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Readobj {
    pub file_summary: FileSummary,
    pub sections: Vec<SectionItem>,
    pub symbols: Vec<SymbolItem>,
}

#[test]
fn test_readobj() {
    let content = crate::util::read_file_as_bytes("file.json").unwrap();
    let content = if content.starts_with(&[0xFF, 0xFE]) {
        let content: &[u16] = bytemuck::cast_slice(content.index(2..));
        String::from_utf16(content).unwrap()
    } else {
        String::from_utf8(content).unwrap()
    };

    let content: Vec<Readobj> = serde_json::from_str(content.as_str()).unwrap();
    println!("{:#?}", content);
}

pub fn read(path: impl AsRef<Path>) -> anyhow::Result<Readobj> {
    let content = exec("llvm-readobj")
        .arg("--elf-output-style=JSON")
        .arg(path.as_ref())
        .arg("--sections")
        .arg("--symbols")
        .exec_and_get_stdout_as_string()
        .context("command failed")?;

    let value: Vec<Readobj> = serde_json::from_str(content.as_str())
        .context("unable to deserialize output")?;
    assert_eq!(value.len(), 1, "found more than one readobj entry");

    Ok(value.into_iter().next().unwrap())
}