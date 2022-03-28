use crate::{
  app::shared_resources::cancel_lock,
  light::{
    color::NormColor,
    controller::{MemoryController, MemoryControllerExt, U32MemoryController},
    Lights,
  },
  util::AsmDelay,
};

use super::Show;

pub struct UniformShow(NormColor);
impl UniformShow {
  pub fn new(color: NormColor) -> Self {
    Self(color)
  }
}

impl Show for UniformShow {
  fn run(&mut self, lights: &mut Lights, asm_delay: AsmDelay, _: &mut cancel_lock) {
    let mut ctrl = U32MemoryController::new(lights, asm_delay);

    ctrl.set_all(self.0);
    ctrl.display();
  }
}
