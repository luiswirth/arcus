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

#[derive(Default)]
pub struct QuickShow;
impl Show for QuickShow {
  fn update(&mut self, lights: &mut Lights, asm_delay: AsmDelay) -> State {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem, asm_delay);

    ctrl.set_all(Color::WHITE);
    ctrl.display();
    State::Running
  }
}
