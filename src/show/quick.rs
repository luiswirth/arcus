use crate::{
  light::{
    color::NormColor,
    controller::{MemoryControllerExt, U32MemoryController},
  },
  return_cancel,
  util::AsmDelay,
};

use super::Show;

#[derive(Default)]
pub struct QuickShow;
impl Show for QuickShow {
  fn run(
    &mut self,
    cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut U32MemoryController,
    _asm_delay: AsmDelay,
    _remote_input: &mut crate::app::shared_resources::remote_input_lock,
    _configuration: &mut crate::app::shared_resources::configuration_lock,
  ) {
    ctrl.set_all(NormColor::RED);
    return_cancel!(cancel);
  }
}
