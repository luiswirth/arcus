use embedded_hal::blocking::delay::DelayMs;

use crate::{
  app::shared_resources::cancel_lock,
  light::{
    color::Color,
    controller::{MemoryController, U32Memory, U32MemoryController},
    Lights,
  },
  util::AsmDelay,
};

use super::Show;
use crate::return_cancel;

pub struct DemoShow {
  memory: U32Memory,
}
impl Default for DemoShow {
  fn default() -> Self {
    Self {
      memory: U32Memory::new(),
    }
  }
}

impl Show for DemoShow {
  fn run(&mut self, lights: &mut Lights, mut asm_delay: AsmDelay, cancel: &mut cancel_lock) {
    const N: usize = Lights::N;
    let mut ctrl = U32MemoryController::new(lights, &mut self.memory, asm_delay);

    let colors = [
      Color::RED,
      Color::GREEN,
      Color::BLUE,
      Color::YELLOW,
      Color::MAGENTA,
      Color::CYAN,
    ];

    loop {
      for color in colors {
        for l in 0..N {
          ctrl.set(l, color);
          ctrl.display();
          asm_delay.delay_ms(16);
          return_cancel!(cancel);
        }
      }
    }
  }
}
