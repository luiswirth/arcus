use arclib::nl;

use crate::{
  light::{
    color::NormRgbw,
    controller::{ColorMemoryController, MemoryController},
    Lights,
  },
  util::AsmDelay,
};

use super::Show;

pub struct GradientShow {
  from: NormRgbw,
  to: NormRgbw,
}
impl GradientShow {
  pub fn new(from: NormRgbw, to: NormRgbw) -> Self {
    Self { from, to }
  }
}
impl Show for GradientShow {
  fn run(
    &mut self,
    _cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut ColorMemoryController,
    _asm_delay: AsmDelay,
    _remote_input: &mut crate::app::shared_resources::remote_input_lock,
    config: &mut crate::app::shared_resources::config_lock,
  ) {
    for l in 0..Lights::N {
      let lf = nl!(l) / nl!(Lights::N - 1);
      ctrl.set(l, self.from.gradient(self.to, lf));
    }
    ctrl.display(config);
  }
}
