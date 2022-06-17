use rtic::Mutex;

use crate::{app::shared_resources::configuration_lock, util::AsmDelay};

use super::{Lights, NormColor};

// TODO: every controller should respect configuration (brightness etc.)

/// Raw Controller.
/// Doesn't have a memory associated.
// TODO: Switch to a iterator instead of array?
pub struct DirectController<'a> {
  lights: &'a mut Lights,
  asm_delay: AsmDelay,
}
impl<'a> DirectController<'a> {
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
  fn display(&mut self);
}
pub trait MemoryControllerExt {
  fn set_range(&mut self, range: core::ops::Range<usize>, color: NormColor);
  fn set_all(&mut self, color: NormColor);
}

/// A memory controller which stores the raw u32 colors.
/// Every get/set needs a conversion, but displays doesn't.
/// Good if only a few colors change between every display.
pub struct U32MemoryController<'a> {
  lights: &'a mut Lights,
  memory: [u32; Lights::N],
  asm_delay: AsmDelay,
}
impl<'a> U32MemoryController<'a> {
  pub fn new(lights: &'a mut Lights, asm_delay: AsmDelay) -> Self {
    Self {
      lights,
      memory: [0; Lights::N],
      asm_delay,
    }
  }
}
impl<'a> MemoryController<'a> for U32MemoryController<'a> {
  fn set(&mut self, i: usize, color: NormColor) {
    self.memory[i] = color.into_u32();
  }

  // TODO: fix or remove this.
  // This dones't seem to work, probably because
  // of the conversions between integers and floats.
  fn get(&self, i: usize) -> NormColor {
    NormColor::from_u32(self.memory[i])
  }

  fn display(&mut self) {
    self
      .lights
      .write_iter(self.memory.into_iter(), self.asm_delay);
  }
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

  pub fn display_with_config(&mut self, config_lock: &mut configuration_lock) {
    let brightness = config_lock.lock(|config| config.brightness);
    self.lights.write_iter(
      self
        .memory
        .into_iter()
        .map(|c| c.scale_rgbw(brightness))
        .map(|c| c.into_u32()),
      self.asm_delay,
    );
  }
}
impl<'a> MemoryController<'a> for ColorMemoryController<'a> {
  fn set(&mut self, i: usize, color: NormColor) {
    self.memory[i] = color;
  }

  fn get(&self, i: usize) -> NormColor {
    self.memory[i]
  }

  fn display(&mut self) {
    self.lights.write_iter(
      self.memory.into_iter().map(|c| c.into_u32()),
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
