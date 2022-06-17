use rtic::Mutex;

use crate::{app::shared_resources::config_lock, util::AsmDelay};

use super::{Lights, NormColor};

/// Raw Controller.
/// Doesn't have a memory associated.
pub struct RawController<'a> {
  lights: &'a mut Lights,
  asm_delay: AsmDelay,
}
impl<'a> RawController<'a> {
  pub fn set_display(&mut self, colors: [u32; Lights::N]) {
    self.lights.write_iter(colors.into_iter(), self.asm_delay);
  }
}

/// Memory Controller trait.
/// Controllers with associated memory.
pub trait MemoryController<'a> {
  const N: usize = Lights::N;

  fn set(&mut self, i: usize, color: NormColor);
  fn get(&self, i: usize) -> NormColor;
  /// Doesn't respect brightness
  fn display(&mut self, config: &mut config_lock);
}
pub trait MemoryControllerExt {
  fn set_range(&mut self, range: core::ops::Range<usize>, color: NormColor);
  fn set_all(&mut self, color: NormColor);
}

/// A memory controller which stores the color type.
/// Every display needs to convert, but get/set doesn't.
/// Good if a lot of colors change between displays.
pub struct ColorMemoryController<'a> {
  lights: &'a mut Lights,
  memory: [NormColor; Lights::N],
  asm_delay: AsmDelay,
}
impl<'a> ColorMemoryController<'a> {
  pub fn new(lights: &'a mut Lights, asm_delay: AsmDelay) -> Self {
    let memory = [NormColor::NONE; Lights::N];
    Self {
      lights,
      memory,
      asm_delay,
    }
  }
}
impl<'a> MemoryController<'a> for ColorMemoryController<'a> {
  fn set(&mut self, i: usize, color: NormColor) {
    self.memory[i] = color;
  }

  fn get(&self, i: usize) -> NormColor {
    self.memory[i]
  }

  fn display(&mut self, config: &mut config_lock) {
    let brightness = config.lock(|config| config.brightness);
    self.lights.write_iter(
      self
        .memory
        .into_iter()
        .map(|c| c.brightness(brightness))
        .map(|c| c.into_u32()),
      self.asm_delay,
    );
  }
}

impl<'a, M> MemoryControllerExt for M
where
  M: MemoryController<'a>,
{
  fn set_range(&mut self, range: core::ops::Range<usize>, color: NormColor) {
    for i in range {
      self.set(i, color);
    }
  }

  fn set_all(&mut self, color: NormColor) {
    for i in 0..Self::N {
      self.set(i, color);
    }
  }
}
