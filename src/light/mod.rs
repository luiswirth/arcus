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

use crate::util::AsmDelay;

use self::color::Color;

pub struct Lights {
  tx: Tx<(PIO0, SM0)>,
}

impl Lights {
  pub const N: usize = 20; //4 * 60;
  pub fn init(
    pio_instance: pac::PIO0,
    resets: &mut pac::RESETS,
    sysclock_freq: f32,
    _pin2: gpio::Pin<gpio::bank0::Gpio2, gpio::FunctionPio0>,
  ) -> Self {
    let side_set = pio::SideSet::new(false, 1, false);

    let mut assembler = pio::Assembler::new_with_side_set(side_set);

    // configure pin as output
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
    assembler.nop_with_delay_and_side_set(T2 - 1, 0);
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

    Self { tx }
  }

  fn force_write(&mut self, word: u32) {
    while !self.tx.write(word) {}
  }

  fn write_iter(&mut self, words: impl Iterator<Item = u32>, mut asm_delay: AsmDelay) {
    for word in words {
      self.force_write(word);
    }
    // TODO: this should be 60us
    asm_delay.delay_us(600);
  }
}
