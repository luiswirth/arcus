use rp_pico::hal::timer::CountDown;

use crate::light::{
  color::Color,
  controller::{MemoryController, MemoryControllerExt, U32Memory, U32MemoryController},
  show::State,
  Lights,
};

use super::Show;

#[derive(Default)]
pub struct QuickShow;
impl Show for QuickShow {
  fn update(&mut self, lights: &mut Lights, count_down: CountDown) -> State {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem, count_down);

    ctrl.set_all(Color::WHITE);
    ctrl.display();
    State::Running
  }
}
