use super::{Color, Lights};

/// Raw Controller.
/// Doesn't have a memory associated.
// TODO: Switch to a iterator instead of array?
pub struct DirectController<'a>(&'a mut Lights);
impl<'a> DirectController<'a> {
  pub fn set_display(&mut self, colors: [u32; Lights::N]) {
    for c in colors {
      self.0.force_write(c);
    }
  }
}

/// Memory Controller trait.
/// Controllers with associated memory.
pub trait MemoryController<'a> {
  const N: usize = Lights::N;

  fn set(&mut self, i: usize, color: Color);
  fn get(&self, i: usize) -> Color;
  fn display(&mut self);
}
pub trait MemoryControllerExt {
  fn set_all(&mut self, color: Color);
}

/// A memory controller which stores the raw u32 colors.
/// Every get/set needs a conversion, but displays doesn't.
/// Good if only a few colors change between every display.
pub struct U32Memory([u32; Lights::N]);
impl U32Memory {
  pub fn new() -> Self {
    let memory = [0u32; Lights::N];
    Self(memory)
  }
}

pub struct U32MemoryController<'a> {
  memory: &'a mut U32Memory,
  lights: &'a mut Lights,
}
impl<'a> U32MemoryController<'a> {
  pub fn new(lights: &'a mut Lights, memory: &'a mut U32Memory) -> Self {
    Self { lights, memory }
  }
}
impl<'a> MemoryController<'a> for U32MemoryController<'a> {
  fn set(&mut self, i: usize, color: Color) {
    self.memory.0[i] = color.into_u32();
  }

  // TODO: fix or remove this.
  // This dones't seem to work, probably because
  // of the conversions between integers and floats.
  fn get(&self, i: usize) -> Color {
    Color::from_u32(self.memory.0[i])
  }

  fn display(&mut self) {
    for c in &self.memory.0 {
      self.lights.force_write(*c);
    }
  }
}

/// A memory controller which stores the color type.
/// Every display needs to convert, but get/set doesn't.
/// Good if a lot of colors change between displays.
pub struct ColorMemoryController<'a> {
  lights: &'a mut Lights,
  memory: [Color; Lights::N],
}
impl<'a> ColorMemoryController<'a> {
  pub fn new(lights: &'a mut Lights) -> Self {
    let memory = [Color::NONE; Lights::N];
    Self { lights, memory }
  }
}
impl<'a> MemoryController<'a> for ColorMemoryController<'a> {
  fn set(&mut self, i: usize, color: Color) {
    self.memory[i] = color;
  }

  fn get(&self, i: usize) -> Color {
    self.memory[i]
  }

  fn display(&mut self) {
    for c in &self.memory {
      self.lights.force_write(c.into_u32());
    }
  }
}

impl<'a, M> MemoryControllerExt for M
where
  M: MemoryController<'a>,
{
  fn set_all(&mut self, color: Color) {
    for i in 0..Self::N {
      self.set(i, color);
    }
  }
}
