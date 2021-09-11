use num_traits::float::Float;
use rand::{Rng, SeedableRng};

use crate::light::{
  color::Color,
  controller::{ColorMemoryController, MemoryController, MemoryControllerExt},
  Lights, Utils,
};

use super::Show;

const N: usize = Lights::N;

pub struct Firefly {
  pos: f32,
  speed: f32,
  hue: f32,
}

impl Firefly {
  pub const PLACEHOLDER: Self = Self {
    pos: 0.0,
    speed: 0.0,
    hue: 0.0,
  };

  pub fn new(rng: &mut impl Rng) -> Self {
    let pos = rng.gen::<f32>();
    let speed = (rng.gen::<f32>() * 2.0 - 1.0) / 100.0;
    let hue = rng.gen::<f32>();
    Self { pos, speed, hue }
  }

  pub fn fly(&mut self) {
    self.pos = self.pos + self.speed;
    if self.pos >= 1.0 {
      self.pos -= 1.0;
    }
    if self.pos < 0.0 {
      self.pos += 1.0;
    }
  }

  pub fn visualize(&self, ctrl: &mut ColorMemoryController) {
    for l in 0..N {
      let lf = l as f32 / N as f32;
      let mut dist = (lf - self.pos).abs();
      dist = dist.min(1.0 - dist); // because fireflys wrap around
      let prox = 1.0 - dist;
      let value = prox.powf(70.0);

      let mut new_color = Color::from_hsv(self.hue, 1.0, value);
      let old_color = ctrl.get(l);
      if old_color != Color::NONE {
        new_color = new_color.mix_rgbw(old_color);
      }
      ctrl.set(l, new_color);
    }
  }
}

// - random movement

pub struct FireflyShow;
impl Show for FireflyShow {
  fn play(&mut self, lights: &mut Lights, utils: &mut Utils) {
    let mut ctrl = ColorMemoryController::new(lights);

    let mut rng = rand::rngs::SmallRng::seed_from_u64(17843938646114006223);

    const NMUGGI: usize = 2;
    let mut muggis = [Firefly::PLACEHOLDER; NMUGGI];
    for i in 0..NMUGGI {
      muggis[i] = Firefly::new(&mut rng);
    }

    loop {
      ctrl.set_all(Color::NONE);
      for muggi in &mut muggis {
        muggi.fly();
        muggi.visualize(&mut ctrl);
      }
      ctrl.display();
      //utils.delay_ms(1000 / 60);
    }
  }
}
