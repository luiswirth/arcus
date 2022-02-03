use crate::light::{
  color::Color,
  controller::{MemoryController, MemoryControllerExt, U32Memory, U32MemoryController},
  show::State,
  Lights,
};

use super::Show;

pub struct UniformShow(Color);
impl UniformShow {
  pub fn new(color: Color) -> Self {
    Self(color)
  }
}

impl Show for UniformShow {
  fn update(&mut self, lights: &mut Lights) -> State {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem);

    ctrl.set_all(self.0);
    ctrl.display();
    State::Finished
  }
}
