use crate::light::{
  color::Color,
  controller::{MemoryController, MemoryControllerExt, U32Memory, U32MemoryController},
  show::State,
  Lights, Utils,
};
use piclib::{ONE, ZERO};

use super::Show;

//const N: usize = Lights::N;
#[derive(Default)]
pub struct QuickShow;
impl Show for QuickShow {
  fn update(&mut self, lights: &mut Lights, _utils: &mut Utils) -> State {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem);

    ctrl.set_all(Color::new(ONE, ZERO, ZERO, ZERO));
    ctrl.display();

    State::Finished
  }
}
