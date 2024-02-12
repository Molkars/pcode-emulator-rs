use std::ffi::OsStr;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter, ErrorKind, Read, Seek, SeekFrom};
use std::path::Path;
use std::process::{Command};

pub fn read_file_as_bytes(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let len = file.seek(SeekFrom::End(0))? as usize;
    file.seek(SeekFrom::Start(0))?;
    let mut reader = BufReader::new(file);
    let mut out = Vec::with_capacity(len);
    reader.read_to_end(&mut out)?;
    Ok(out)
}

#[inline]
#[allow(unused)]
pub fn write_to_file(path: impl AsRef<Path>, f: impl FnOnce(&mut BufWriter<File>) -> io::Result<()>) -> io::Result<()> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    f(&mut writer)?;
    Ok(())
}

#[inline]
pub fn exec(s: impl AsRef<OsStr>) -> Command {
    Command::new(s)
}

pub trait ExecUtil {
    fn exec_and_get_stdout_as_string(self) -> io::Result<String>;
}

impl<'a> ExecUtil for &'a mut Command {
    fn exec_and_get_stdout_as_string(self) -> io::Result<String> {
        let output = self.output()?;
        if !output.status.success() {
            Err(io::Error::new(ErrorKind::Other, ""))
        } else {
            String::from_utf8(output.stdout)
                .map_err(|e| io::Error::new(
                    ErrorKind::Other,
                    format!("unable to run `{:?}`: {}", self, e),
                ))
        }
    }
}

impl ExecUtil for Command {
    #[inline]
    fn exec_and_get_stdout_as_string(mut self) -> io::Result<String> {
        <&mut Self as ExecUtil>::exec_and_get_stdout_as_string(&mut self)
    }
}