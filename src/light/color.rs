use arclib::{nl, Fix32, ONE, ZERO};

pub type RawColor = u32;
pub type RawChannel = u8;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NormRgbw {
  pub r: Fix32,
  pub g: Fix32,
  pub b: Fix32,
  pub w: Fix32,
}

impl NormRgbw {
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NormHsv {
  pub hue: Fix32,
  pub sat: Fix32,
  pub val: Fix32,
}

impl NormHsv {
  pub fn new(hue: Fix32, sat: Fix32, val: Fix32) -> Self {
    Self { hue, sat, val }
  }

  pub fn mix(self, other: Self) -> Self {
    self.gradient(other, ONE / nl!(2))
  }

  pub fn gradient(self, other: Self, t: Fix32) -> Self {
    Self::new(
      (ONE - t) * self.hue + t * other.hue,
      (ONE - t) * self.sat + t * other.sat,
      (ONE - t) * self.val + t * other.val,
    )
  }
}

impl From<NormHsv> for NormRgbw {
  #[allow(clippy::zero_prefixed_literal)]
  fn from(hsv: NormHsv) -> Self {
    let NormHsv { mut hue, sat, val } = hsv;
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
}

impl From<NormRgbw> for NormHsv {
  fn from(norm_color: NormRgbw) -> Self {
    let [r, g, b] = [norm_color.r, norm_color.g, norm_color.b];
    let cmax = r.max(g).max(b);
    let cmin = r.min(g).min(b);
    let delta = cmax - cmin;

    let hue = if delta == ZERO {
      ZERO
    } else {
      nl!(60) / nl!(360)
        * match cmax {
          c if c == r => ((g - b) / delta) % nl!(6),
          c if c == g => (b - r) / delta + nl!(2),
          c if c == b => (r - g) / delta + nl!(4),
          _ => unreachable!(),
        }
    };
    let sat = if cmax == ZERO { ZERO } else { delta / cmax };
    let val = cmax;

    Self::new(hue, sat, val)
  }
}

/// Color definitions
impl NormRgbw {
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

  pub const STANDARD_PALETTE: [NormRgbw; 9] = [
    NormRgbw::RED,
    NormRgbw::GREEN,
    NormRgbw::BLUE,
    NormRgbw::WHITE,
    NormRgbw::RGB,
    NormRgbw::RGBW,
    NormRgbw::YELLOW,
    NormRgbw::MAGENTA,
    NormRgbw::CYAN,
  ];
}

impl core::ops::Index<usize> for NormRgbw {
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
impl core::ops::IndexMut<usize> for NormRgbw {
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

impl rand::distributions::Distribution<NormHsv> for rand::distributions::Standard {
  fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> NormHsv {
    let hue = nl!(rng.gen::<f32>());
    NormHsv::new(hue, ONE, ONE)
  }
}

impl NormRgbw {
  /// Brightness `b` between 0.0 and 1.0
  #[must_use]
  pub fn brightness(self, mut b: Fix32) -> Self {
    b = (cordic::exp(b) - nl!(1)) / (nl!(fixed::consts::E) - nl!(1));
    self.scale(b)
  }

  #[must_use]
  pub fn scale(self, scalar: Fix32) -> Self {
    NormRgbw::new(
      scalar * self.r,
      scalar * self.g,
      scalar * self.b,
      scalar * self.w,
    )
  }

  #[must_use]
  pub fn mix(self, other: Self) -> Self {
    self.gradient(other, ONE / nl!(2))
  }

  #[must_use]
  pub fn gradient(self, other: Self, t: Fix32) -> Self {
    NormRgbw::new(
      (ONE - t) * self.r + t * other.r,
      (ONE - t) * self.g + t * other.g,
      (ONE - t) * self.b + t * other.b,
      (ONE - t) * self.w + t * other.w,
    )
  }
}

impl core::ops::Add for NormRgbw {
  type Output = NormRgbw;
  fn add(self, other: Self) -> Self {
    NormRgbw::new(
      self.r + other.r,
      self.g + other.g,
      self.b + other.b,
      self.w + other.w,
    )
  }
}
