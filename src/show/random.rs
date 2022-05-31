use rand::{Rng, SeedableRng};

use crate::{
  app::monotonics,
  light::{controller::MemoryController, Lights},
  return_cancel,
};

use super::Show;

#[derive(Default)]
pub struct RandomShow;

impl Show for RandomShow {
  fn run(
    &mut self,
    cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut crate::light::controller::U32MemoryController,
    _asm_delay: crate::util::AsmDelay,
    _remote_input: &mut crate::app::shared_resources::remote_input_lock,
    _configuration: &mut crate::app::shared_resources::configuration_lock,
  ) {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(monotonics::now().ticks());
    loop {
      for l in 0..Lights::N {
        ctrl.set(l, rng.gen());
      }
      ctrl.display();
      return_cancel!(cancel);
    }
  }
}
