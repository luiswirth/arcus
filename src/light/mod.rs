pub mod color;
pub mod controller;
pub mod show;

use cortex_m::delay::Delay;
use pico_explorer::{
  hal::{self, gpio},
  pac,
};

use crate::light::controller::{MemoryController, U32MemoryController};

use self::color::Color;

pub struct Lights {
  pio: hal::pio::PIO<pac::PIO0>,
}

impl Lights {
  pub const N: usize = 60;
  pub fn init(
    pio_instance: pac::PIO0,
    pin2: gpio::Pin<gpio::bank0::Gpio2, gpio::FunctionPio0>,
    resets: &mut pac::RESETS,
    sysclock_freq: f32,
  ) -> Self {
    let side_set = pio::SideSet::new(true, 1, false);

    let mut assembler = pio::Assembler::new_with_side_set(side_set);

    // configure pin as output
    // TODO: is this really necessary for side_set?
    assembler.set(pio::SetDestination::PINDIRS, 1);

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

    let program = assembler.assemble(Some((wrap_source, wrap_target)));

    let pio = hal::pio::PIO::new(pio_instance, resets);
    let sm = &pio.state_machines()[0];

    const FREQ: f32 = 8_000_000.0;
    let div = sysclock_freq / FREQ;

    let builder = hal::pio::PIOBuilder::default()
      .with_program(&program)
      .buffers(hal::pio::Buffers::OnlyTx)
      .out_shift_direction(hal::pio::ShiftDirection::Left)
      .autopull(true)
      .pull_threshold(32)
      .clock_divisor(div)
      .set_pins(2, 1)
      .side_set(side_set)
      .side_set_pin_base(2);

    builder.build(&pio, sm).unwrap();
    sm.set_enabled(true);

    Self { pio }
  }

  fn sm(&self) -> &hal::pio::StateMachine<pac::PIO0> {
    &self.pio.state_machines()[0]
  }

  // TODO: block instead of retry
  fn sm_force_push(&self, value: u32) {
    while !self.sm().write_tx(value) {}
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
