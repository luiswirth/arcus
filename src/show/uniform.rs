use crate::{
  light::{
    color::NormColor,
    controller::{MemoryController, MemoryControllerExt},
  },
  return_cancel,
};

use super::Show;

pub struct UniformShow(NormColor);
impl UniformShow {
  pub fn new(color: NormColor) -> Self {
    Self(color)
  }
}

impl Show for UniformShow {
  fn run(
    &mut self,
    cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut crate::light::controller::ColorMemoryController,
    _asm_delay: crate::util::AsmDelay,
    _remote_input: &mut crate::app::shared_resources::remote_input_lock,
    config: &mut crate::app::shared_resources::config_lock,
  ) {
    // TODO: remove busy loop
    loop {
      ctrl.set_all(self.0);
      ctrl.display(config);
      return_cancel!(cancel);
    }
  }
}
