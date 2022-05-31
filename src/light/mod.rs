pub mod color;
pub mod controller;

use cortex_m::prelude::_embedded_hal_blocking_delay_DelayUs;
use rp_pico::{
  hal::{
    self, gpio,
    pac::PIO0,
    pio::{PIOExt, Tx, SM0},
  },
  pac,
};

use self::color::NormColor;
use crate::util::AsmDelay;

pub struct Lights {
  tx: Tx<(PIO0, SM0)>,
}

pub type LightsPin = gpio::Pin<gpio::bank0::Gpio2, gpio::FunctionPio0>;
const LIGHTS_PIN_IDX: u8 = 2;

impl Lights {
  pub const N: usize = 4 * 60;
  pub fn init(
    pio_instance: pac::PIO0,
    resets: &mut pac::RESETS,
    sysclock_freq: f32,
    _lights_pin: LightsPin,
  ) -> Self {
    let side_set = pio::SideSet::new(false, 1, false);

    let mut assembler = pio::Assembler::new_with_side_set(side_set);

    const T1: u8 = 2;
    const T2: u8 = 5;
    const T3: u8 = 3;
    const CYCLES_PER_BIT: u32 = (T1 + T2 + T3) as u32;
    const FREQ: u32 = 800_000;

    let mut wrap_target = assembler.label();
    let mut do_one = assembler.label();
    let mut do_zero = assembler.label();
    let mut wrap_source = assembler.label();

    assembler.bind(&mut wrap_target);
    assembler.out_with_delay_and_side_set(pio::OutDestination::X, 1, T3 - 1, 0);
    assembler.jmp_with_delay_and_side_set(pio::JmpCondition::XIsZero, &mut do_zero, T1 - 1, 1);
    assembler.bind(&mut do_one);
    assembler.jmp_with_delay_and_side_set(pio::JmpCondition::Always, &mut wrap_target, T2 - 1, 1);
    assembler.bind(&mut do_zero);
    assembler.nop_with_delay_and_side_set(T2 - 1, 0);
    assembler.bind(&mut wrap_source);

    let program = assembler.assemble_with_wrap(wrap_source, wrap_target);
    let (mut pio, sm, _, _, _) = pio_instance.split(resets);
    let installed = pio.install(&program).unwrap();

    let div = sysclock_freq / (FREQ as f32 * CYCLES_PER_BIT as f32);

    let (mut sm, _, tx) = hal::pio::PIOBuilder::from_program(installed)
      .buffers(hal::pio::Buffers::OnlyTx)
      .out_shift_direction(hal::pio::ShiftDirection::Left)
      .autopull(true)
      .pull_threshold(32)
      .clock_divisor(div)
      .side_set_pin_base(2)
      .build(sm);

    sm.set_pindirs([(LIGHTS_PIN_IDX, hal::pio::PinDir::Output)]);

    sm.start();

    Self { tx }
  }

  fn write_iter(&mut self, words: impl Iterator<Item = u32>, mut asm_delay: AsmDelay) {
    for word in words {
      // idle write until the fifo isn't full anymore
      while !self.tx.write(word) {}
    }
    // TODO: instead of wait, start a countdown and wait on the next iteration
    // wait until fifo is empty
    while !self.tx.is_empty() {}
    asm_delay.delay_us(80);
  }
}
