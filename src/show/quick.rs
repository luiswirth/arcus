use crate::{
  app::shared_resources::cancel_lock,
  light::{
    color::NormColor,
    controller::{MemoryControllerExt, U32MemoryController},
    Lights,
  },
  return_cancel,
  util::AsmDelay,
};

use super::Show;

#[derive(Default)]
pub struct QuickShow;
impl Show for QuickShow {
  fn run(&mut self, lights: &mut Lights, asm_delay: AsmDelay, cancel: &mut cancel_lock) {
    let mut ctrl = U32MemoryController::new(lights, asm_delay);
    ctrl.set_all(NormColor::RED);
    return_cancel!(cancel);
  }
}
