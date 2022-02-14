use crate::{
  light::{
    color::Color,
    controller::{MemoryController, MemoryControllerExt, U32Memory, U32MemoryController},
    Lights,
  },
  show::State,
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
  fn update(&mut self, lights: &mut Lights, asm_delay: AsmDelay) -> State {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem, asm_delay);

    ctrl.set_all(self.0);
    ctrl.display();
    State::Finished
  }
}
