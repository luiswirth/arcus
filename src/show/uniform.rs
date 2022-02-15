use crate::{
  app::shared_resources::cancel_lock,
  light::{
    color::Color,
    controller::{MemoryController, MemoryControllerExt, U32Memory, U32MemoryController},
    Lights,
  },
  util::AsmDelay,
};

use super::Show;

pub struct UniformShow(Color);
impl UniformShow {
  pub fn new(color: Color) -> Self {
    Self(color)
  }
}

impl Show for UniformShow {
  fn run(&mut self, lights: &mut Lights, asm_delay: AsmDelay, _: &mut cancel_lock) {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem, asm_delay);

    ctrl.set_all(self.0);
    ctrl.display();
  }
}
