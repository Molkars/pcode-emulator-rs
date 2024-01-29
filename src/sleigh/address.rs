use std::marker::PhantomData;
use std::pin::Pin;

pub struct Address<'a> {
    inner: Pin<Box<&'a super::ffi::Address>>,
}

impl<'a> From<&'a super::ffi::Address> for Address<'a> {
    fn from(value: &'a crate::sleigh::ffi::Address) -> Self {
        Self {
            inner: Box::pin(value),
        }
    }
}