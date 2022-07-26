use core::ops::Add;

use cortex_m::prelude::*;

use crate::{
  light::{
    color::NormRgbw,
    controller::{ColorMemoryController, MemoryController, MemoryControllerExt},
    Lights,
  },
  return_cancel,
  util::AsmDelay,
};

use super::Show;

#[derive(Default)]
pub struct RgbClockShow {
  with_seconds: bool,
}
impl Show for RgbClockShow {
  fn run(
    &mut self,
    cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut ColorMemoryController,
    mut asm_delay: AsmDelay,
    _remote_input: &mut crate::app::shared_resources::remote_input_lock,
    config: &mut crate::app::shared_resources::config_lock,
  ) {
    const N24: usize = Lights::N / 24;
    const N60: usize = Lights::N / 60;

    for hour in 19..24 {
      for minute in 12..60 {
        for second in 0..60 {
          for l in 0..Lights::N {
            let mut color = NormRgbw::NONE;
            if l < (hour + 1) * N24 {
              color = color.add(NormRgbw::RED);
            }
            if l < (minute + 1) * N60 {
              color = color.add(NormRgbw::GREEN);
            }
            if self.with_seconds && l < (second + 1) * N60 {
              color = color.add(NormRgbw::BLUE);
            }
            ctrl.set(l, color);
          }
          ctrl.display(config);
          asm_delay.delay_ms(500);
          return_cancel!(cancel);
        }
      }
    }
  }
}

#[derive(Default)]
pub struct SeparatedClockShow;
impl Show for SeparatedClockShow {
  fn run(
    &mut self,
    cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut ColorMemoryController,
    mut asm_delay: AsmDelay,
    _remote_input: &mut crate::app::shared_resources::remote_input_lock,
    config: &mut crate::app::shared_resources::config_lock,
  ) {
    let cn = Lights::N / 2;
    let n12 = cn / 12;
    let n60 = cn / 60;

    for hour in 0..12 {
      for minute in 0..60 {
        ctrl.set_all(NormRgbw::NONE);
        ctrl.set_range(0..(n12 * hour), NormRgbw::RED);
        ctrl.set_range(cn..(cn + n60 * minute), NormRgbw::GREEN);

        for i in 0..12 {
          let l = i * n12;
          let color = if i % 3 == 0 {
            NormRgbw::BLUE
          } else {
            NormRgbw::BLUE.mix(NormRgbw::WHITE)
          };
          ctrl.set(l, color);
          ctrl.set(cn + l, color);
        }

        ctrl.display(config);
        asm_delay.delay_ms(100);
        return_cancel!(cancel);
      }
    }
  }
}
