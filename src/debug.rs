use core::{fmt::Write, panic::PanicInfo};

use embedded_graphics::{
  mono_font::{MonoFont, MonoTextStyleBuilder},
  pixelcolor::Rgb565,
  prelude::*,
  text::{Alignment, Baseline, LineHeight, Text, TextStyleBuilder},
};
use embedded_hal::digital::v2::OutputPin;
use pico_explorer::{
  hal::gpio::{bank0::Gpio25, Output, Pin, PushPull},
  Screen,
};

pub type Led = Pin<Gpio25, Output<PushPull>>;
pub type ArrayString = arrayvec::ArrayString<{ NCHARS as usize }>;

pub const SCREEN_SIZE: u16 = 240;
pub const SCREEN_SIZE2: usize = SCREEN_SIZE as usize * SCREEN_SIZE as usize;

static mut SCREEN: Option<Screen> = None;
static mut LED: Option<Led> = None;

fn usage() {
  //let led = pins.led.into_push_pull_output();

  //debug::init_debug(led, explorer.screen);
  //let mut string = debug::ArrayString::new();

  //let string = debug::breakup(string);
  //debug::sprint(&string);
}

pub fn led<'a>() -> &'a Led {
  unsafe { LED.as_ref().unwrap() }
}

pub fn led_mut<'a>() -> &'a mut Led {
  unsafe { LED.as_mut().unwrap() }
}

pub fn screen<'a>() -> &'a Screen {
  unsafe { SCREEN.as_ref().unwrap() }
}
pub fn screen_mut<'a>() -> &'a mut Screen {
  unsafe { SCREEN.as_mut().unwrap() }
}

pub fn init_debug(led: Led, screen: Screen) {
  unsafe {
    LED = Some(led);
  }
  unsafe {
    SCREEN = Some(screen);
  }
}

const FONT: MonoFont = embedded_graphics::mono_font::ascii::FONT_4X6;

const CHAR_WIDTH: u32 = FONT.character_size.width + FONT.character_spacing;
const CHAR_HEIGHT: u32 = FONT.character_size.height;
const CHARS_PER_ROW: u32 = SCREEN_SIZE as u32 / CHAR_WIDTH;
const CHARS_PER_COL: u32 = SCREEN_SIZE as u32 / CHAR_HEIGHT;
const NCHARS: u32 = CHARS_PER_ROW * CHARS_PER_COL;

pub fn breakup(s: ArrayString) -> ArrayString {
  let mut r = ArrayString::new();
  let mut last = 0usize;
  for (i, c) in s.char_indices() {
    r.push(c);
    if c == '\n' {
      last = i;
    } else if i - last >= CHARS_PER_ROW as usize {
      r.push('\n');
      last = i;
    }
  }
  r
}

pub fn sprint(text: &str) {
  screen_mut().clear(Rgb565::BLACK).unwrap();
  draw_text(text);
}

pub fn draw_text(text: &str) {
  let char_style = MonoTextStyleBuilder::new()
    .font(&FONT)
    .text_color(Rgb565::GREEN)
    .background_color(Rgb565::BLACK)
    //.reset_background_color()
    .build();

  let text_style = TextStyleBuilder::new()
    .alignment(Alignment::Left)
    .baseline(Baseline::Top)
    .line_height(LineHeight::Pixels(6))
    .build();

  Text::with_text_style(text, Point::new(0, 0), char_style, text_style)
    .draw(screen_mut())
    .unwrap();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
  led_mut().set_high().unwrap();

  let mut buf = ArrayString::new();
  writeln!(buf, "{}", info).unwrap();
  buf = breakup(buf);

  screen_mut().clear(Rgb565::RED).unwrap();
  draw_text(&buf);

  loop {}
}
