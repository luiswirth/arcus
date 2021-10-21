use crate::light::{
  color::Color,
  controller::{MemoryController, U32Memory, U32MemoryController},
  show::State,
  Lights, Utils,
};

use super::Show;

const N: usize = Lights::N;
pub struct GradientShow;
impl Show for GradientShow {
  fn update(&mut self, lights: &mut Lights, utils: &mut Utils) -> State {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem);

    let colors = [Color::YELLOW, Color::RED];

    let np = N / (colors.len() - 1);
    let npf = np as f32;

    for (i, cs) in colors.windows(2).enumerate() {
      if let [c0, c1] = *cs {
        for l in 0..np {
          let lf = l as f32 / npf;
          let c01 = c0.gradient_rgbw(c1, lf);
          ctrl.set(i * np + l, c01);
        }
      }
    }
    loop {
      ctrl.display();
      utils.delay_ms(1000);
    }
    State::Finished
  }
}
