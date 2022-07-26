use crate::{
  light::{
    color::NormRgbw,
    controller::{ColorMemoryController, MemoryControllerExt},
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
    ctrl: &mut ColorMemoryController,
    _asm_delay: AsmDelay,
    _remote_input: &mut crate::app::shared_resources::remote_input_lock,
    _config: &mut crate::app::shared_resources::config_lock,
  ) {
    ctrl.set_all(NormRgbw::RED);
    return_cancel!(cancel);
  }
}
