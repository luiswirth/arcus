use crate::light::{
  color::NormColor,
  controller::{MemoryController, MemoryControllerExt},
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
    _cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut crate::light::controller::U32MemoryController,
    _asm_delay: crate::util::AsmDelay,
    _remote_input: &mut crate::app::shared_resources::remote_input_lock,
    _configuration: &mut crate::app::shared_resources::configuration_lock,
  ) {
    ctrl.set_all(self.0);
    ctrl.display();
  }
}
