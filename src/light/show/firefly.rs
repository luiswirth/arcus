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
  vel: f32,
  acc: f32,
  hue: f32,
}

impl Firefly {
  pub const PLACEHOLDER: Self = Self {
    pos: 0.0,
    vel: 0.0,
    acc: 0.0,
    hue: 0.0,
  };

  pub fn new(rng: &mut impl Rng) -> Self {
    let pos = rng.gen::<f32>();
    let vel = 0.;
    let acc = 0.;
    let hue = rng.gen::<f32>();
    Self { pos, vel, acc, hue }
  }

  pub fn fly(&mut self, rng: &mut impl Rng) {
    self.acc += (rng.gen::<f32>() * 2. - 1.) / 10000.;
    self.pos += self.vel;
    if self.pos >= 1. {
      self.pos -= 1.;
    }
    if self.pos < 0. {
      self.pos += 1.;
    }

    self.hue += (rng.gen::<f32>() * 2. - 1.) / 100.;
    if self.hue >= 1. {
      self.pos -= 1.;
    }
    if self.hue < 0. {
      self.hue += 1.
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

pub struct FireflyShow;
impl Show for FireflyShow {
  fn play(&mut self, lights: &mut Lights, _utils: &mut Utils) {
    let mut ctrl = ColorMemoryController::new(lights);

    let mut rng = rand::rngs::SmallRng::seed_from_u64(17843938646114006223);

    const NMUGGI: usize = 2;
    let mut muggis = [Firefly::PLACEHOLDER; NMUGGI];
    for muggi in muggis.iter_mut() {
      *muggi = Firefly::new(&mut rng);
    }

    loop {
      ctrl.set_all(Color::NONE);
      for muggi in &mut muggis {
        muggi.fly(&mut rng);
        muggi.visualize(&mut ctrl);
      }
      ctrl.display();
      //utils.delay_ms(1000 / 60);
    }
  }
}
