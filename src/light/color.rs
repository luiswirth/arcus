use arclib::{nl, Fix32, ONE, ZERO};

pub type RawColor = u32;
pub type RawChannel = u8;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
  pub r: Fix32,
  pub g: Fix32,
  pub b: Fix32,
  pub w: Fix32,
}

impl Color {
  pub const fn new(r: Fix32, g: Fix32, b: Fix32, w: Fix32) -> Self {
    Self { r, g, b, w }
  }

  pub const fn into_channel_array(self) -> [Fix32; 4] {
    [self.r, self.g, self.b, self.w]
  }
  pub const fn from_channel_array(channels: [Fix32; 4]) -> Self {
    Self::new(channels[0], channels[1], channels[2], channels[3])
  }

  pub fn into_u8_channel_array(self) -> [u8; 4] {
    denormalize(self.into_channel_array())
  }
  pub fn from_u8_channel_array(channels: [u8; 4]) -> Self {
    Self::from_channel_array(normalize(channels))
  }

  pub fn into_u32(self) -> u32 {
    pack(self.into_u8_channel_array())
  }
  pub fn from_u32(value: u32) -> Self {
    Self::from_u8_channel_array(unpack(value))
  }

  #[allow(clippy::zero_prefixed_literal)]
  pub fn from_hsv(mut hue: Fix32, sat: Fix32, val: Fix32) -> Self {
    hue *= nl!(360u16);
    let c = val * sat;
    let v = (hue / nl!(60u16)) % nl!(2u16) - ONE;
    let v = if v < ZERO { -v } else { v };
    let x = c * (ONE - v);
    let m = val - c;
    let (r, g, b) = if hue < nl!(60u16) {
      (c, x, ZERO)
    } else if hue < nl!(120u16) {
      (x, c, ZERO)
    } else if hue < nl!(180u16) {
      (ZERO, c, x)
    } else if hue < nl!(240u16) {
      (ZERO, x, c)
    } else if hue < nl!(300u16) {
      (x, ZERO, c)
    } else {
      (c, ZERO, x)
    };
    Self::new(r + m, g + m, b + m, ZERO)
  }

  pub fn into_hsv(self) -> [Fix32; 3] {
    unimplemented!()
  }
}

/// Color definitions
impl Color {
  pub const RED: Self = Self::new(ONE, ZERO, ZERO, ZERO);
  pub const GREEN: Self = Self::new(ZERO, ONE, ZERO, ZERO);
  pub const BLUE: Self = Self::new(ZERO, ZERO, ONE, ZERO);
  pub const WHITE: Self = Self::new(ZERO, ZERO, ZERO, ONE);

  pub const RGB: Self = Self::new(ONE, ONE, ONE, ZERO);
  pub const RGBW: Self = Self::new(ONE, ONE, ONE, ONE);
  pub const NONE: Self = Self::new(ZERO, ZERO, ZERO, ZERO);

  pub const YELLOW: Self = Self::new(ONE, ONE, ZERO, ZERO);
  pub const MAGENTA: Self = Self::new(ONE, ZERO, ONE, ZERO);
  pub const CYAN: Self = Self::new(ZERO, ONE, ONE, ZERO);

  pub const STANDARD_PALETTE: [Color; 9] = [
    Color::RED,
    Color::GREEN,
    Color::BLUE,
    Color::WHITE,
    Color::RGB,
    Color::RGBW,
    Color::YELLOW,
    Color::MAGENTA,
    Color::CYAN,
  ];
}

impl core::ops::Index<usize> for Color {
  type Output = Fix32;

  fn index(&self, index: usize) -> &Self::Output {
    match index {
      0 => &self.r,
      1 => &self.g,
      2 => &self.b,
      3 => &self.w,
      _ => panic!("Out of range"),
    }
  }
}
impl core::ops::IndexMut<usize> for Color {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    match index {
      0 => &mut self.r,
      1 => &mut self.g,
      2 => &mut self.b,
      3 => &mut self.w,
      _ => panic!("Out of range"),
    }
  }
}

pub fn normalize([r, g, b, w]: [u8; 4]) -> [Fix32; 4] {
  [
    nl!(r) / nl!(255u8),
    nl!(g) / nl!(255u8),
    nl!(b) / nl!(255u8),
    nl!(w) / nl!(255u8),
  ]
}

pub fn denormalize([r, g, b, w]: [Fix32; 4]) -> [u8; 4] {
  [
    (r * nl!(255u8)).to_num(),
    (g * nl!(255u8)).to_num(),
    (b * nl!(255u8)).to_num(),
    (w * nl!(255u8)).to_num(),
  ]
}

#[allow(clippy::many_single_char_names)]
#[allow(clippy::identity_op)]
pub fn pack([r, g, b, w]: [u8; 4]) -> u32 {
  let mut grbw = 0u32;
  grbw |= (g as u32) << 24;
  grbw |= (r as u32) << 16;
  grbw |= (b as u32) << 8;
  grbw |= (w as u32) << 0;
  grbw
}

#[allow(clippy::many_single_char_names)]
#[allow(clippy::identity_op)]
pub fn unpack(c: u32) -> [u8; 4] {
  let g = ((c | 0xFF_00_00_00) >> 24) as u8;
  let r = ((c | 0x00_FF_00_00) >> 16) as u8;
  let b = ((c | 0x00_00_FF_00) >> 8) as u8;
  let w = ((c | 0x00_00_00_FF) >> 0) as u8;
  [r, g, b, w]
}

impl rand::distributions::Distribution<Color> for rand::distributions::Standard {
  fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Color {
    let hue = nl!(rng.gen::<f32>());
    Color::from_hsv(hue, ONE, ONE)
  }
}

impl Color {
  #[must_use]
  pub fn scale_rgbw(self, scalar: Fix32) -> Self {
    Color::new(
      scalar * self.r,
      scalar * self.g,
      scalar * self.b,
      scalar * self.w,
    )
  }

  #[must_use]
  pub fn add_rgbw(self, other: Self) -> Self {
    Color::new(
      self.r + other.r,
      self.g + other.g,
      self.b + other.b,
      self.w + other.w,
    )
  }

  #[must_use]
  pub fn mix_rgbw(self, other: Self) -> Self {
    self.gradient_rgbw(other, ONE / nl!(2))
  }

  //pub fn mix_hsv(self, other: Self) -> Self {
  //  let this = self.into_hsv();
  //  let other = other.into_hsv();
  //  let h = (this[0] + other[0]) / nl!(2);
  //  let s = (this[1] + other[1]) / nl!(2);
  //  let v = (this[2] + other[2]) / nl!(2);
  //  Color::from_hsv(h, s, v)
  //}

  #[must_use]
  pub fn gradient_rgbw(self, other: Self, t: Fix32) -> Self {
    Color::new(
      (ONE - t) * self.r + t * other.r,
      (ONE - t) * self.g + t * other.g,
      (ONE - t) * self.b + t * other.b,
      (ONE - t) * self.w + t * other.w,
    )
  }
  //pub fn gradient_hsv(self, other: Self, t: FixNorm) -> Self {
  //  let this = self.into_hsv();
  //  Color::from_hsv(
  //    (ONE - t) * this[0] + t * other.r,
  //    (ONE - t) * this[1] + t * other.g,
  //    (ONE - t) * this[2] + t * other.b,
  //  )
  //}
}
