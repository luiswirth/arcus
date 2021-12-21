use crate::light::{
  color::Color,
  controller::{MemoryController, MemoryControllerExt, U32Memory, U32MemoryController},
  show::State,
  Lights, Utils,
};
use arclib::{nl, FixNorm, ONE};

use super::Show;

const N: usize = Lights::N;
#[derive(Default)]
pub struct QuickShow {
  i: u32,
}
impl Show for QuickShow {
  fn update(&mut self, lights: &mut Lights, _utils: &mut Utils) -> State {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem);
    // 2 red 2 blue 2 yellow 2 magenta 2 green

    ctrl.set_all(Color::RED);
    ctrl.display();

    for l in 0..N {
      let lf = nl!(l) / nl!(N);
      let hue = lf + nl!(self.i) / nl!(100u32);
      let c = Color::from_hsv(hue % ONE, ONE, ONE);
      ctrl.set(l, c);
      //ctrl.set(i, match i % 10 {
      //    0..=1 => Color::GREEN,
      //    2..=3 => Color::BLUE,
      //    4..=5 => Color::YELLOW,
      //    6..=7 => Color::MAGENTA,
      //    8..=9 => Color::RED,
      //    _ => unreachable!(),
      //});
    }
    ctrl.set_all(Color::MAGENTA);
    ctrl.display();

    self.i += 1;
    State::Running
  }
}
