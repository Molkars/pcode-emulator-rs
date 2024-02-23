use std::sync::{Arc, Mutex, TryLockError, TryLockResult};
use std::thread::spawn;

pub struct Task<T> {
    value: Arc<Mutex<Option<T>>>,
}

impl<T> Task<T> {
    pub fn new<F>(f: F) -> Self
        where F: FnOnce() -> T + Send + 'static,
              T: Send + 'static
    {
        let value = Arc::new(Mutex::new(None));
        let value_clone = value.clone();
        spawn(move || {
            let value = f();
            value_clone.as_ref()
                .lock()
                .as_deref_mut()
                .expect("lock poisoned")
                .replace(value);
        });
        Self {
            value,
        }
    }
}

pub trait TaskHandle<T> {
    fn get_result(&mut self) -> Option<T>;
}

impl<T> TaskHandle<T> for Option<Task<T>> {
    fn get_result(&mut self) -> Option<T> {
        let result = if let Some(task) = self.as_ref() {
            match task.value.as_ref().try_lock() {
                Ok(mut value) => value.take(),
                Err(TryLockError::WouldBlock) => None,
                Err(TryLockError::Poisoned(_)) => panic!("lock poisoned"),
            }
        } else {
            None
        }?;
        *self = None;
        Some(result)
    }
}