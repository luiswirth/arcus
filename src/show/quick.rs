use arclib::{nl, ZERO};
use cortex_m::prelude::*;

use crate::{
  app::shared_resources::cancel_lock,
  light::{
    color::Color,
    controller::{MemoryController, MemoryControllerExt, U32Memory, U32MemoryController},
    Lights,
  },
  return_cancel,
  util::AsmDelay,
};

use super::Show;

#[derive(Default)]
pub struct QuickShow;
impl Show for QuickShow {
  fn run(&mut self, lights: &mut Lights, mut asm_delay: AsmDelay, cancel: &mut cancel_lock) {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem, asm_delay);

    for brightness in 0..256u32 {
      let brightness = nl!(brightness) / nl!(255u32);
      ctrl.set_all(Color::new(brightness, ZERO, ZERO, ZERO));
      ctrl.display();
    }

    //loop {
    //  ctrl.set_all(Color::RED);
    //  for l in 0..Lights::N {
    //    ctrl.set(l, Color::NONE);
    //    ctrl.display();
    //    return_cancel!(cancel);
    //  }
    //}

    //for size in 1..Lights::N {
    //  for color in Color::STANDARD_PALETTE {
    //    for front_pos in 0..(Lights::N - size) {
    //      for infront_pos in 0..front_pos {
    //          ctrl.set(infront_pos, Color::NONE);
    //      }
    //      for offset in 0..size {
    //        ctrl.set(front_pos + offset, color);
    //      }
    //      //ctrl.set(l.saturating_sub(1), Color::NONE);
    //      ctrl.display();
    //      asm_delay.delay_us(250);
    //      return_cancel!(cancel);
    //    }
    //    ctrl.set(Lights::N - 1, Color::NONE);
    //  }
    //}
  }
}
