use crate::light::{
  color::Color,
  controller::{MemoryController, MemoryControllerExt, U32Memory, U32MemoryController},
  show::State,
  Lights, Utils,
};

use super::Show;

pub struct OffShow;
impl Show for OffShow {
  fn update(&mut self, lights: &mut Lights, _utils: &mut Utils) -> State {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem);
    ctrl.set_all(Color::NONE);
    ctrl.display();
    State::Finished
  }
}
