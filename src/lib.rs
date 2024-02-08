#![allow(unused)]

use serde::Serialize;

mod util;
#[cfg(debug_assertions)]
mod inspector;
pub mod symbol_table;
pub mod emulator;
pub mod binary;


#[cfg(debug_assertions)]
#[inline]
pub fn inspect<T: Serialize>(value: &T) {
    inspector::inspect(value)
}

#[cfg(not(debug_assertions))]
pub fn inspect<T: Serialize>(value: &T) {}