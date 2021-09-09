use cortex_m::delay::Delay;
use palette::{rgb::Rgb, IntoColor};
use pico_explorer::{
  hal::{self, gpio},
  pac,
};

pub struct Lights {
  pio: hal::pio::PIO<pac::PIO0>,
  delay: Delay,
}

impl Lights {
  pub const N: usize = 60;
  pub fn init(
    pio_instance: pac::PIO0,
    pin2: gpio::Pin<gpio::bank0::Gpio2, gpio::FunctionPio0>,
    resets: &mut pac::RESETS,
    sysclock_freq: f32,
    delay: Delay,
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

    Self { pio, delay }
  }

  pub fn sm(&self) -> &hal::pio::StateMachine<pac::PIO0> {
    &self.pio.state_machines()[0]
  }

  // TODO: block instead of retry
  pub fn sm_force_push(&self, value: u32) {
    while !self.sm().write_tx(value) {}
  }

  pub fn play_lightshow(&mut self, ls: &mut dyn Lightshow) {
    let ctrl = MemoryController::new(self);
    ls.play(ctrl)
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

pub fn pack([r, g, b, w]: [u8; 4]) -> u32 {
  let mut grbw = 0u32;
  grbw |= (g as u32) << 24;
  grbw |= (r as u32) << 16;
  grbw |= (b as u32) << 8;
  grbw |= (w as u32) << 0;
  grbw
}

pub fn unpack(c: u32) -> [u8; 4] {
  let g = ((c | 0xFF_00_00_00) >> 24) as u8;
  let r = ((c | 0x00_FF_00_00) >> 16) as u8;
  let b = ((c | 0x00_00_FF_00) >> 8) as u8;
  let w = ((c | 0x00_00_00_FF) >> 0) as u8;
  [r, g, b, w]
}

/// Contrary to palette hue is 0..1 instead of 0..360
fn hsv2rgbw(hsv: [f32; 3]) -> [f32; 4] {
  let hsv = palette::Hsv::new(hsv[0] * 360.0, hsv[1], hsv[2]);
  let rgb: Rgb = hsv.into_color();
  let comps = rgb.into_components();
  [comps.0, comps.1, comps.2, 0.0]
}

#[allow(dead_code)]
struct RawController<'a>(&'a mut Lights);
#[allow(dead_code)]
impl<'a> RawController<'a> {
  const N: usize = Lights::N;
  pub fn display(&mut self, colors: [u32; Lights::N]) {
    for c in colors {
      self.0.sm_force_push(c);
    }
  }
}

pub struct MemoryController<'a> {
  lights: &'a mut Lights,
  memory: [u32; Lights::N],
}
impl<'a> MemoryController<'a> {
  const N: usize = Lights::N;

  fn new(lights: &'a mut Lights) -> Self {
    let memory = [0u32; Lights::N];
    Self { lights, memory }
  }

  pub fn set(&mut self, i: usize, rgbw: [f32; 4]) {
    self.memory[i] = pack(denormalize(rgbw));
  }

  #[allow(dead_code)]
  pub fn get(&self, i: usize) -> [f32; 4] {
    normalize(unpack(self.memory[i]))
  }

  pub fn display(&self) {
    for c in &self.memory {
      self.lights.sm_force_push(*c);
    }
  }

  pub fn delay_ms(&mut self, ms: u32) {
    self.lights.delay.delay_ms(ms);
  }
  #[allow(dead_code)]
  pub fn delay_us(&mut self, us: u32) {
    self.lights.delay.delay_us(us);
  }
}
trait MemoryControllerExt {
  fn set_all(&mut self, rgbw: [f32; 4]);
}
impl<'a> MemoryControllerExt for MemoryController<'a> {
  fn set_all(&mut self, rgbw: [f32; 4]) {
    let rgbw = pack(denormalize(rgbw));
    for c in &mut self.memory {
      *c = rgbw;
    }
  }
}

// TODO: let LightShow choose with controller they want to use
pub trait Lightshow {
  fn play(&mut self, ctrl: MemoryController);
}

pub struct ExampleLightshow {}
impl Lightshow for ExampleLightshow {
  fn play(&mut self, mut ctrl: MemoryController) {
    const N: usize = MemoryController::N;
    loop {
      ctrl.set_all([0.0; 4]);

      for comp in 0..4 {
        let mut color = [0f32; 4];
        color[comp] = 1.0;
        for l in 0..N {
          ctrl.set(l, color);
          ctrl.display();
          ctrl.delay_ms(40);
        }
      }

      let color = [1f32; 4];
      for l in 0..N {
        ctrl.set(l, color);
        ctrl.display();
        ctrl.delay_ms(40);
      }
      ctrl.set_all([0.0; 4]);

      for pass in 0..2 {
        for l in 0..N {
          let hue = l as f32 / N as f32;
          let color = hsv2rgbw([hue, 1.0, 1.0]);
          if pass == 0 && l != 0 {
            ctrl.set(l - 1, [0.0; 4]);
          }
          ctrl.set(l, color);
          ctrl.display();
          ctrl.delay_ms(40);
        }
      }

      let mut hue = 0.0;
      loop {
        hue += 0.01;
        if hue > 1.0 {
          break;
        }
        for l in 0..N {
          let color = hsv2rgbw([hue, 1.0, 1.0]);
          ctrl.set(l, color);
        }
        ctrl.display();
        ctrl.delay_ms(40);
      }
    }
  }
}
