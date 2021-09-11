use crate::light::{
  color::Color,
  controller::{MemoryController, MemoryControllerExt, U32MemoryController},
  Lights, Utils,
};

use super::Show;

pub struct DemoShow;
impl Show for DemoShow {
  fn play(&mut self, lights: &mut Lights, utils: &mut Utils) {
    const N: usize = Lights::N;
    let mut ctrl = U32MemoryController::new(lights);
    loop {
      ctrl.set_all(Color::NONE);

      for comp in 0..4 {
        let mut color = Color::NONE;
        color[comp] = 1.0;
        for l in 0..N {
          ctrl.set(l, color);
          ctrl.display();
          utils.delay_ms(40);
        }
      }

      let color = Color::ALL;
      for l in 0..N {
        ctrl.set(l, color);
        ctrl.display();
        utils.delay_ms(40);
      }
      ctrl.set_all(Color::NONE);

      for pass in 0..2 {
        for l in 0..N {
          let hue = l as f32 / N as f32;
          let color = Color::from_hsv(hue, 1.0, 1.0);
          if pass == 0 && l != 0 {
            ctrl.set(l - 1, Color::NONE);
          }
          ctrl.set(l, color);
          ctrl.display();
          utils.delay_ms(40);
        }
      }

      let mut hue = 0.0;
      loop {
        hue += 0.01;
        if hue > 1.0 {
          break;
        }
        for l in 0..N {
          let color = Color::from_hsv(hue, 1.0, 1.0);
          ctrl.set(l, color);
        }
        ctrl.display();
        utils.delay_ms(40);
      }
    }
  }
}
