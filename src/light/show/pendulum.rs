use core::f32::consts::TAU;
use num_traits::Float;

use crate::light::{
  color::Color,
  controller::{MemoryController, U32Memory, U32MemoryController},
  show::State,
  Lights, Utils,
};

use super::Show;

const N: usize = Lights::N;
const NM: f32 = (Lights::N - 1) as f32;
pub struct PendulumShow;
impl Show for PendulumShow {
  fn update(&mut self, lights: &mut Lights, utils: &mut Utils) -> State {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem);

    let period = 500f32;
    let freq0 = period.recip();
    let delta = 2.;

    let mut time = 0.;
    loop {
      let mut pos_arr = [0f32; 3];
      for (c, pos) in pos_arr.iter_mut().enumerate() {
        let freq = freq0 * (c as f32 + 1.);
        *pos = ((time * freq * TAU).sin() + 1.) / 2.;
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
    State::Finished
  }
}
