#![allow(dead_code)]

use crate::shared::Shared;

pub fn slice_len<T>(a: &[T], b: &[T]) -> bool {
    a.len() == b.len()
}

pub fn shared_slice_len<T>(a: &Shared<Vec<T>>, b: &Shared<Vec<T>>) -> bool {
    a.read().len() == b.read().len()
}

pub fn shared_error_eq(a: &Shared<Vec<anyhow::Error>>, b: &Shared<Vec<anyhow::Error>>) -> bool {
    let a = a.read();
    let b = b.read();
    if a.len() != b.len() {
        return false;
    }

    let a = a.iter();
    let b = b.iter();
    a.zip(b).all(|(a, b)| a.root_cause().to_string() == b.root_cause().to_string())
}