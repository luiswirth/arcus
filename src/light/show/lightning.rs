use alloc::vec::Vec;
use rand::{Rng, SeedableRng};

use crate::light::{
  color::Color,
  controller::{MemoryController, MemoryControllerExt, U32Memory, U32MemoryController},
  show::State,
  Lights, Utils,
};

use super::Show;

const N: usize = Lights::N;

pub struct SparkleShow;
impl Show for SparkleShow {
  fn update(&mut self, lights: &mut Lights, utils: &mut Utils) -> State {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem);

    let mut rng = rand::rngs::SmallRng::seed_from_u64(17843938646114006223);

    loop {
      ctrl.set_all(Color::NONE);
      let origin = rng.gen_range(0..N);
      //let origin = N / 2;
      let color0 = rng.gen::<Color>();
      let color1 = rng.gen::<Color>();
      let low_range = (0..origin).rev();
      let high_range = origin..N;
      let len = low_range.len().min(high_range.len());
      for pos2 in low_range.zip(high_range) {
        let pos2 = [pos2.0, pos2.1];
        for pos in pos2 {
          let dist = (pos as isize - origin as isize).abs() as f32 / len as f32;
          let color = color0.gradient_hsv(color1, dist);
          ctrl.set(pos, color);
        }
        ctrl.display();
        utils.delay_ms(100);
      }
    }
    State::Finished
  }
}

enum Direction {
  Down,
  Up,
}
struct Particle {
  pos: isize,
  prev_pos: isize,
  direction: Direction,
  hue: f32,
}
struct Explosion {
  origin: isize,
  distance: isize,
  hue: f32,
}

impl Particle {
  fn new(rng: &mut impl Rng) -> Self {
    let pos = rng.gen_range(0..N as isize);
    let hue = rng.gen::<f32>();
    let direction = if rng.gen::<bool>() {
      Direction::Down
    } else {
      Direction::Up
    };
    Self {
      pos,
      prev_pos: pos,
      direction,
      hue,
    }
  }
}
impl Explosion {
  fn new(origin: isize, hue: f32) -> Self {
    Self {
      origin,
      distance: 0,
      hue,
    }
  }
}

const SPAWN_PROBABILITY: f32 = 1. / 10.;

pub struct CollisionShow;
impl Show for CollisionShow {
  fn update(&mut self, lights: &mut Lights, utils: &mut Utils) -> State {
    let mut mem = U32Memory::new();
    let mut ctrl = U32MemoryController::new(lights, &mut mem);

    let mut rng = rand::rngs::SmallRng::seed_from_u64(17843938646114006223);
    let mut particles = Vec::new();
    let mut explosions = Vec::new();

    loop {
      ctrl.set_all(Color::NONE);
      if rng.gen::<f32>() < SPAWN_PROBABILITY {
        particles.push(Particle::new(&mut rng));
      }

      for p in &mut particles {
        match p.direction {
          Direction::Down => p.pos -= 1,
          Direction::Up => p.pos += 1,
        };
      }
      particles.retain(|p| {
        let color = Color::from_hsv(p.hue, 1., 1.);

        if (0..N as isize).contains(&p.pos) {
          ctrl.set(p.pos as usize, color);
          true
        } else {
          false
        }
      });

      product_retain(&mut particles, |pi, pj| {
        let prev_sign = (pi.prev_pos - pj.prev_pos).signum();
        let curr_sign = (pi.pos - pj.pos).signum();
        let exploded = prev_sign != curr_sign;
        if exploded {
          let mid = (pi.pos + pj.pos) / 2;
          let mix = (pi.hue + pj.hue) / 2.;
          explosions.push(Explosion::new(mid, mix));
          false
        } else {
          true
        }
      });

      for e in &mut explosions {
        e.distance += 1;
      }

      explosions.retain(|e| {
        let pos0 = e.origin - e.distance;
        let pos1 = e.origin + e.distance;
        let color = Color::from_hsv(e.hue, 1., 0.1);
        let mut used = false;
        let bounds = 0..(N as isize);
        if bounds.contains(&pos0) {
          ctrl.set(pos0 as usize, color);
          used = true;
        }
        if bounds.contains(&pos1) {
          ctrl.set(pos1 as usize, color);
          used = true;
        }
        used
      });

      ctrl.display();
      utils.delay_ms(50);
    }
    State::Finished
  }
}

fn product_retain<T, F>(v: &mut Vec<T>, mut pred: F)
where
  F: FnMut(&T, &T) -> bool,
{
  let mut j = 0;
  for i in 0..v.len() {
    // invariants:
    // items v[0..j] will be kept
    // items v[j..i] will be removed
    if (0..j).chain(i + 1..v.len()).all(|a| pred(&v[i], &v[a])) {
      v.swap(i, j);
      j += 1;
    }
  }
  v.truncate(j);
}
