use core::f32::consts::TAU;
use num_traits::Float;

use crate::light::{
  color::Color,
  controller::{MemoryController, U32MemoryController},
  Lights, Utils,
};

use super::Show;

const N: usize = Lights::N;
const NM: f32 = (Lights::N - 1) as f32;
pub struct PendulumShow;
impl Show for PendulumShow {
  fn play(&mut self, lights: &mut Lights, utils: &mut Utils) {
    let mut ctrl = U32MemoryController::new(lights);

    let period = 5000f32;
    let freq0 = period.recip();
    let delta = 2.;

    let mut time = 0.;
    loop {
      let mut pos_arr = [0f32; 3];
      for c in 0..3 {
        let freq = freq0 * (c as f32 + 1.);
        let pos = ((time * freq * TAU).sin() + 1.) / 2.;
        pos_arr[c] = pos;
      }
      for l in 0..N {
        let lf = l as f32 / NM;
        let mut color = Color::NONE;
        for c in 0..3 {
          let dist = (pos_arr[c] - lf).abs();
          let prox = 1. - dist;
          let intensity = prox.powf(100.);
          color[c] = intensity;
        }
        ctrl.set(l, color);
      }

      ctrl.display();
      utils.delay_ms(delta as u32);
      time += delta;
    }
  }
}
