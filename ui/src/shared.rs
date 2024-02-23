use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use druid::{Data, Lens};

pub trait ReadCloned {
    type Output;

    fn read_cloned(&self) -> Self::Output;
}

impl<T> ReadCloned for Shared<Option<T>>
    where T: Clone
{
    type Output = Option<T>;

    fn read_cloned(&self) -> Self::Output {
        self.read().clone()
    }
}

#[derive(Default, Debug, Data)]
pub struct Shared<T>(Rc<RefCell<T>>);

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared(self.0.clone())
    }
}

impl<T> Shared<T> {
    #[inline]
    pub fn read(&self) -> std::cell::Ref<T> {
        self.0.borrow()
    }

    #[inline]
    pub fn write(&self) -> std::cell::RefMut<T> {
        self.0.borrow_mut()
    }

    pub fn write_in_place<O>(&self, f: impl FnOnce(&mut T) -> O) -> O {
        f(self.0.borrow_mut().deref_mut())
    }

    pub fn read_in_place<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        f(self.0.borrow().deref())
    }

    #[inline]
    pub fn new(value: T) -> Self {
        Shared(Rc::new(RefCell::new(value)))
    }

    pub fn replace(&self, value: T) -> T {
        let mut borrow = self.0.borrow_mut();
        std::mem::replace(&mut *borrow, value)
    }
}

impl<T> Shared<Option<T>> {
    #[inline]
    pub fn take(&self) -> Option<T> {
        self.write_in_place(|value| value.take())
    }
}