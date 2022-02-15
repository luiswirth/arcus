use rand::{Rng, SeedableRng};

use crate::{
  app::{monotonics, shared_resources::cancel_lock},
  light::{
    controller::{MemoryController, U32Memory, U32MemoryController},
    Lights,
  },
  return_cancel,
  util::AsmDelay,
};

use super::Show;

#[derive(Default)]
pub struct RandomShow;

impl Show for RandomShow {
  fn run(&mut self, lights: &mut Lights, asm_delay: AsmDelay, cancel: &mut cancel_lock) {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem, asm_delay);

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
