pub type RawColor = u32;
pub type RawChannel = u8;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
  pub r: f32,
  pub g: f32,
  pub b: f32,
  pub w: f32,
}

impl Color {
  pub const fn new(r: f32, g: f32, b: f32, w: f32) -> Self {
    Self { r, g, b, w }
  }

  pub const fn into_channel_array(self) -> [f32; 4] {
    [self.r, self.g, self.b, self.w]
  }
  pub const fn from_channel_array(channels: [f32; 4]) -> Self {
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

  pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
    let hsv = palette::Hsv::new(h * 360.0, s, v);
    let rgb: palette::rgb::Rgb = palette::IntoColor::into_color(hsv);
    let comps = rgb.into_components();
    Self::new(comps.0, comps.1, comps.2, 0.0)
  }

  pub fn into_hsv(self) -> [f32; 3] {
    let rgb = palette::rgb::Rgb::new(self.r, self.g, self.b);
    let rgb: palette::Hsv = palette::IntoColor::into_color(rgb);
    let comps = rgb.into_components();
    [comps.0.to_positive_degrees() / 360.0, comps.1, comps.2]
  }
}

/// Color definitions
impl Color {
  pub const RED: Self = Self::new(1.0, 0.0, 0.0, 0.0);
  pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 0.0);
  pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 0.0);
  pub const WHITE: Self = Self::new(0.0, 0.0, 0.0, 1.0);

  pub const ALL: Self = Self::new(1.0, 1.0, 1.0, 1.0);
  pub const NONE: Self = Self::new(0.0, 0.0, 0.0, 0.0);

  pub const YELLOW: Self = Self::RED.mix_rgbw(Self::GREEN);
  pub const MAGENTA: Self = Self::RED.mix_rgbw(Self::BLUE);
  pub const CYAN: Self = Self::GREEN.mix_rgbw(Self::BLUE);

  pub const ORANGE: Self = Self::RED.mix_rgbw(Self::YELLOW);
}

impl core::ops::Index<usize> for Color {
  type Output = f32;

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

pub fn normalize([r, g, b, w]: [u8; 4]) -> [f32; 4] {
  [
    r as f32 / 255.0,
    g as f32 / 255.0,
    b as f32 / 255.0,
    w as f32 / 255.0,
  ]
}

pub fn denormalize([r, g, b, w]: [f32; 4]) -> [u8; 4] {
  [
    (r * 255.0) as u8,
    (g * 255.0) as u8,
    (b * 255.0) as u8,
    (w * 255.0) as u8,
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
    let hue = rng.gen::<f32>();
    Color::from_hsv(hue, 1.0, 1.0)
  }
}

impl Color {
  pub fn scale_rgbw(self, scalar: f32) -> Self {
    Color::new(
      scalar * self.r,
      scalar * self.g,
      scalar * self.b,
      scalar * self.w,
    )
  }

  pub fn add_rgbw(self, other: Self) -> Self {
    Color::new(
      self.r + other.r,
      self.g + other.g,
      self.b + other.b,
      self.w + other.w,
    )
  }

  pub const fn mix_rgbw(self, other: Self) -> Self {
    self.gradient_rgbw(other, 0.5)
  }

  pub fn mix_hsv(self, other: Self) -> Self {
    let this = self.into_hsv();
    let other = other.into_hsv();
    let h = (this[0] + other[0]) / 2.0;
    let s = (this[1] + other[1]) / 2.0;
    let v = (this[2] + other[2]) / 2.0;
    Color::from_hsv(h, s, v)
  }

  pub const fn gradient_rgbw(self, other: Self, t: f32) -> Self {
    Color::new(
      (1.0 - t) * self.r + t * other.r,
      (1.0 - t) * self.g + t * other.g,
      (1.0 - t) * self.b + t * other.b,
      (1.0 - t) * self.w + t * other.w,
    )
  }
  pub fn gradient_hsv(self, other: Self, t: f32) -> Self {
    let this = self.into_hsv();
    Color::from_hsv(
      (1.0 - t) * this[0] + t * other.r,
      (1.0 - t) * this[1] + t * other.g,
      (1.0 - t) * this[2] + t * other.b,
    )
  }
}
