use alloc::boxed::Box;
use arclib::{Fix32, ONE};

use crate::show::Show;

pub struct Configuration {
  pub show: Option<Box<dyn Show + Send>>,
  pub brightness: Fix32,
}

impl Default for Configuration {
  fn default() -> Self {
    Self {
      show: None,
      brightness: ONE,
    }
  }
}
