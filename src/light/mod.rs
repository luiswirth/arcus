pub mod color;
pub mod controller;
pub mod show;

use cortex_m::delay::Delay;
use embedded_hal::prelude::*;
use embedded_time::duration::Extensions;
use pico_explorer::{
  hal::{
    self, gpio,
    pac::{PIO0, TIMER},
    pio::{PIOExt, Tx, SM0},
    timer::{CountDown, Timer},
  },
  pac,
};
use smart_leds_trait::SmartLedsWrite;

use self_cell::self_cell;

use self::color::Color;

// self-referential cell
self_cell!(
  struct CountDownCell {
    owner: Timer,

    #[covariant]
    dependent: CountDown,
  }
);

pub struct Lights {
  tx: Tx<(PIO0, SM0)>,
  count_down: CountDownCell,
}

impl Lights {
  pub const N: usize = 60 * 4;
  pub fn init(
    pio_instance: pac::PIO0,
    resets: &mut pac::RESETS,
    sysclock_freq: f32,
    _pin2: gpio::Pin<gpio::bank0::Gpio2, gpio::FunctionPio0>,
    timer: TIMER,
  ) -> Self {
    let side_set = pio::SideSet::new(false, 1, false);

    let mut assembler = pio::Assembler::new_with_side_set(side_set);

    // configure pin as output
    // TODO: is this really necessary for side_set?
    assembler.set_with_side_set(pio::SetDestination::PINDIRS, 1, 0);

    let mut wrap_target = assembler.label();
    let mut bitloop = assembler.label();
    let mut do_one = assembler.label();
    let mut do_zero = assembler.label();
    let mut wrap_source = assembler.label();

    const T1: u8 = 2;
    const T2: u8 = 5;
    const T3: u8 = 3;

    assembler.bind(&mut wrap_target);
    assembler.bind(&mut bitloop);
    assembler.out_with_delay_and_side_set(pio::OutDestination::X, 1, T3 - 1, 0);
    assembler.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1);
    assembler.bind(&mut do_one);
    assembler.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut bitloop, T2 - 1, 1);
    assembler.bind(&mut do_zero);
    // Pseudoinstruction: NOP
    assembler.mov_with_delay_and_side_set(
      pio::MovDestination::Y,
      pio::MovOperation::None,
      pio::MovSource::Y,
      T2 - 1,
      0,
    );
    assembler.bind(&mut wrap_source);

    let program = assembler.assemble_with_wrap(wrap_source, wrap_target);

    let (mut pio, sm, _, _, _) = pio_instance.split(resets);

    let installed = pio.install(&program).unwrap();

    const FREQ: f32 = 8_000_000.0;
    let div = sysclock_freq / FREQ;

    let (sm, _, tx) = hal::pio::PIOBuilder::from_program(installed)
      .buffers(hal::pio::Buffers::OnlyTx)
      .out_shift_direction(hal::pio::ShiftDirection::Left)
      .autopull(true)
      .pull_threshold(32)
      .clock_divisor(div)
      .set_pins(2, 1)
      .side_set_pin_base(2)
      .build(sm);

    sm.start();

    let timer = Timer::new(timer, resets);
    let count_down = CountDownCell::new(timer, Timer::count_down);

    Self { tx, count_down }
  }

  // TODO: block instead of retry
  fn force_write(&mut self, word: u32) {
    while !self.tx.write(word) {}
  }
}

// TODO: maybe use more complete crate: `smart_leds` instead of `smart_leds_trait`
impl SmartLedsWrite for Lights {
  type Color = smart_leds_trait::RGBA<u8>;
  type Error = ();

  fn write<T, I>(&mut self, iterator: T) -> Result<(), Self::Error>
  where
    T: Iterator<Item = I>,
    I: Into<Self::Color>,
  {
    self.count_down.with_dependent_mut(|_, c| {
      let _ = nb::block!(c.wait());
    });

    for color in iterator {
      let color: Self::Color = color.into();

      let mut grbw = 0u32;
      grbw |= (color.g as u32) << 24;
      grbw |= (color.r as u32) << 16;
      grbw |= (color.b as u32) << 8;
      grbw |= (color.a as u32) << 0;

      self.force_write(grbw);
    }

    self.count_down.with_dependent_mut(|_, c| {
      c.start(60.microseconds());
    });
    Ok(())
  }
}

/// Utils for controlling the lights.
pub struct Utils {
  delay: Delay,
}
impl Utils {
  pub fn new(delay: Delay) -> Self {
    Self { delay }
  }

  pub fn delay_ms(&mut self, ms: u32) {
    self.delay.delay_ms(ms);
  }
  pub fn delay_us(&mut self, us: u32) {
    self.delay.delay_us(us);
  }
}
