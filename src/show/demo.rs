use arclib::{nl, ONE};
use embedded_hal::blocking::delay::DelayMs;

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
use crate::return_cancel;

#[derive(Default)]
pub struct DemoShow;

impl Show for DemoShow {
  fn run(&mut self, lights: &mut Lights, mut asm_delay: AsmDelay, cancel: &mut cancel_lock) {
    let mut ctrl = U32MemoryController::new(lights, asm_delay);

    ctrl.set_all(NormColor::NONE);

    loop {
      // all colors loading bar
      for color in NormColor::STANDARD_PALETTE {
        for l in 0..Lights::N {
          ctrl.set(l, color);
          ctrl.display();
          asm_delay.delay_ms(2);
          return_cancel!(cancel);
        }
        for l in 0..Lights::N {
          ctrl.set(l, NormColor::NONE);
          ctrl.display();
          asm_delay.delay_ms(2);
          return_cancel!(cancel);
        }
        for brightness in 0..256u32 {
          let brightness = nl!(brightness) / nl!(255u32);
          ctrl.set_all(color.scale_rgbw(brightness));
          ctrl.display();
        }
        ctrl.set_all(NormColor::NONE);
        ctrl.display();
      }
      for shift in 0..360u32 {
        let shiftf = nl!(shift) / nl!(360u32 - 1);
        for l in 0..Lights::N {
          let lf = nl!(l) / nl!(Lights::N - 1);
          let hue = (shiftf + lf).rem_euclid(ONE);
          let color = NormColor::from_hsv(hue, ONE, ONE);
          ctrl.set(l, color);
        }
        ctrl.display();
        asm_delay.delay_ms(16);
        return_cancel!(cancel);
      }
    }
  }
}
