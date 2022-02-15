use crate::{
  app::{
    self,
    show_task::{self, SharedResources},
  },
  light::Lights,
  util::AsmDelay,
};

use rp_pico::{
  hal::{self, clocks::ClockSource, gpio},
  pac,
};
use rtic::Mutex;

pub mod demo;
pub mod quick;
pub mod uniform;

pub use demo::DemoShow;
pub use quick::QuickShow;
pub use uniform::UniformShow;

pub type LightsPin = gpio::Pin<gpio::bank0::Gpio2, gpio::FunctionPio0>;

pub struct ShowTask {
  lights: Lights,
  asm_delay: AsmDelay,
}

impl ShowTask {
  pub fn init(
    lights_pin: LightsPin,
    pio0: pac::PIO0,
    sys_clock: &hal::clocks::SystemClock,
    resets: &mut pac::RESETS,
  ) -> Self {
    let lights = Lights::init(pio0, resets, sys_clock.get_freq().0 as f32, lights_pin);
    let asm_delay = AsmDelay::new(sys_clock.get_freq().0);

    show_task::spawn().unwrap();

    Self { lights, asm_delay }
  }
}

pub fn show_task(ctx: show_task::Context) {
  let ShowTask { lights, asm_delay } = ctx.local.show_task;
  let SharedResources {
    mut show,
    mut cancel,
  } = ctx.shared;
  let show_take = show.lock(|show| show.take());
  if let Some(mut show_take) = show_take {
    cancel.lock(|cancel| cancel.reset());
    Show::run(show_take.as_mut(), lights, *asm_delay, &mut cancel);
  }

  show_task::spawn().unwrap();
}

pub trait Show {
  fn run(
    &mut self,
    lights: &mut Lights,
    asm_delay: AsmDelay,
    cancel: &mut app::shared_resources::cancel_lock,
  );
}

#[derive(Default)]
pub struct CancellationToken {
  requested: bool,
}
impl CancellationToken {
  fn is_requested(&self) -> bool {
    self.requested
  }

  pub fn request(&mut self) {
    self.requested = true;
  }

  fn reset(&mut self) {
    self.requested = false;
  }
}

#[macro_export]
macro_rules! return_cancel {
  ($cancel:ident) => {{
    if ::rtic::Mutex::lock($cancel, |cancel| cancel.is_requested()) {
      return;
    }
  }};
}
