use crate::light::{
  color::Color,
  controller::{MemoryController, U32MemoryController},
  Lights, Utils,
};

use super::Show;

const N: usize = Lights::N;
pub struct GradientShow;
impl Show for GradientShow {
  fn play(&mut self, lights: &mut Lights, _utils: &mut Utils) {
    let mut ctrl = U32MemoryController::new(lights);

    let colors = [Color::RED, Color::BLUE, Color::RED, Color::BLUE];
    let np = N / (colors.len() - 1);
    let npf = np as f32;

    for (i, cs) in colors.windows(2).enumerate() {
      if let &[c0, c1] = cs {
        for l in 0..np {
          let lf = l as f32 / npf;
          let c01 = c0.gradient_rgbw(c1, lf);
          ctrl.set(i * np + l, c01);
        }
      }
    }
    ctrl.display();
  }
}
