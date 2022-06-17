use arclib::{nl, ONE};
use embedded_hal::blocking::delay::DelayMs;

use crate::{
  light::{
    color::NormColor,
    controller::{MemoryController, MemoryControllerExt, U32MemoryController},
    Lights,
  },
  util::AsmDelay,
};

use super::{GradientShow, Show};
use crate::return_cancel;

#[derive(Default)]
pub struct DemoShow;

impl Show for DemoShow {
  fn run(
    &mut self,
    cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut U32MemoryController,
    mut asm_delay: AsmDelay,
    remote_input: &mut crate::app::shared_resources::remote_input_lock,
    configuration: &mut crate::app::shared_resources::configuration_lock,
  ) {
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

      GradientShow::new([NormColor::RED, NormColor::YELLOW]).run(
        cancel,
        ctrl,
        asm_delay,
        remote_input,
        configuration,
      );
    }
  }
}
