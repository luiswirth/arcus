use arclib::nl;

use crate::{
  app::shared_resources::cancel_lock,
  light::{
    color::NormColor,
    controller::{MemoryController, U32MemoryController},
    Lights,
  },
  return_cancel,
  util::AsmDelay,
};

use super::Show;

pub struct GradientShow([NormColor; 2]);
impl GradientShow {
  pub fn new(colors: [NormColor; 2]) -> Self {
    Self(colors)
  }
}
impl Show for GradientShow {
  fn run(&mut self, lights: &mut Lights, asm_delay: AsmDelay, cancel: &mut cancel_lock) {
    let mut ctrl = U32MemoryController::new(lights, asm_delay);
    for l in 0..Lights::N {
      let lf = nl!(l) / nl!(Lights::N - 1);
      ctrl.set(l, self.0[0].gradient_hsv(self.0[1], lf));
    }
    ctrl.display();
    return_cancel!(cancel);
  }
}
