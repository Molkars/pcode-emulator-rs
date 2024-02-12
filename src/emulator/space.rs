use std::cell::{Ref, RefCell};
use std::collections::BTreeMap;

#[derive(Default, Debug)]
pub struct Space {
    /// whether the space is big endian or little endian
    #[allow(unused)]
    big_endian: bool,
    /// a map of address to byte
    inner: RefCell<BTreeMap<u64, u8>>,
    /// an owned buffer to use as temporary storage for get_bytes
    buffer: RefCell<Vec<u8>>,
}

impl Space {
    pub fn new(big_endian: bool) -> Self {
        Self {
            big_endian,
            inner: RefCell::default(),
            buffer: RefCell::new(vec![0; 4]),
        }
    }

    pub fn get_bytes(&self, addr: u64, size: u64) -> Ref<[u8]> {
        let inner = self.inner.borrow_mut();
        let mut buffer = self.buffer.borrow_mut();
        buffer.resize(size as usize, 0u8); // fill the rest with 0

        let start = addr;
        let end = start + size;
        let mut last_key = start;
        // fill the buffer with the bytes from the map, we manually fill the gaps with 0
        for (key, value) in inner.range(start..end) {
            buffer[0..(key - last_key) as usize].fill(0u8);
            buffer[(key - start) as usize] = *value;
            last_key = key + 1;
        }
        buffer.resize(size as usize, 0u8); // fill the rest with 0
        // fill the rest of the buffer with 0
        buffer[(last_key - start) as usize..size as usize].fill(0u8);
        drop(buffer);

        Ref::map(self.buffer.borrow(), |vec| &vec[..size as usize])
    }

    pub fn set_bytes(&self, addr: u64, bytes: &[u8]) {
        let mut inner = self.inner.borrow_mut();
        let start = addr;
        for (i, byte) in bytes.iter().enumerate() {
            inner.insert(start + i as u64, *byte);
        }
    }
}
