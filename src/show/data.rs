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

pub struct ByteShow<'a>(&'a [u8]);
impl<'a> ByteShow<'a> {
  pub fn new(data: &'a [u8]) -> Self {
    Self(data)
  }
}
impl Show for ByteShow<'_> {
  fn run(&mut self, lights: &mut Lights, asm_delay: AsmDelay, _cancel: &mut cancel_lock) {
    let mut ctrl = U32MemoryController::new(lights, asm_delay);

    let nbytes = self.0.len();
    let nbits = 8 * nbytes;
    let nspace = (8 + 1) * nbytes;
    let _lper_bit = Lights::N / nbits;
    let lper_space = Lights::N / nspace;

    ctrl.set_all(NormColor::NONE);
    for ispace in 0..nspace {
      let is_seperator = ispace % 9 == 8;
      let color = if is_seperator {
        NormColor::NONE
      } else {
        let byte = self.0[ispace / 9];
        let bit = ispace % 9;
        let bit = byte & (1 << bit);
        if bit != 0 {
          NormColor::GREEN
        } else {
          NormColor::RED
        }
      };
      ctrl.set_range((ispace * lper_space)..((ispace + 1) * lper_space), color);
      ctrl.display();
    }
  }
}
