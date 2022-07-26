use crate::{
  light::{
    color::NormRgbw,
    controller::{ColorMemoryController, MemoryController, MemoryControllerExt},
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
  fn run(
    &mut self,
    _cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut ColorMemoryController,
    _asm_delay: AsmDelay,
    _remote_input: &mut crate::app::shared_resources::remote_input_lock,
    config: &mut crate::app::shared_resources::config_lock,
  ) {
    let nbytes = self.0.len();
    let nbits = 8 * nbytes;
    let nspace = (8 + 1) * nbytes;
    let _lper_bit = Lights::N / nbits;
    let lper_space = Lights::N / nspace;

    ctrl.set_all(NormRgbw::NONE);
    for ispace in 0..nspace {
      let is_seperator = ispace % 9 == 8;
      let color = if is_seperator {
        NormRgbw::NONE
      } else {
        let byte = self.0[ispace / 9];
        let bit = ispace % 9;
        let bit = byte & (1 << bit);
        if bit != 0 {
          NormRgbw::GREEN
        } else {
          NormRgbw::RED
        }
      };
      ctrl.set_range((ispace * lper_space)..((ispace + 1) * lper_space), color);
      ctrl.display(config);
    }
  }
}
