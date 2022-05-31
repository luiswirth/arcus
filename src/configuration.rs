use alloc::boxed::Box;

use crate::show::Show;

#[derive(Default)]
pub struct Configuration {
  pub show: Option<Box<dyn Show + Send>>,
}
