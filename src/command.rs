use std::process::{Output};

pub trait CommandUtil {
    fn expect_success(&self);
}

impl CommandUtil for Output {
    fn expect_success(&self) {
        if !self.status.success() {
            eprintln!("stderr:\n-------\n:{}",
                String::from_utf8_lossy(self.stdout.as_slice()));
            eprintln!("stdout:\n-------\n:{}",
                String::from_utf8_lossy(self.stderr.as_slice()));
            eprintln!("program exited with failure");
        }
    }
}