use crate::{
  app::{
    self,
    show_task::{self, SharedResources},
  },
  light::{controller::ColorMemoryController, Lights, LightsPin},
  util::AsmDelay,
};

use rp_pico::{
  hal::{self, clocks::ClockSource},
  pac,
};
use rtic::Mutex;

pub mod clock;
pub mod data;
pub mod demo;
pub mod gradient;
pub mod null;
pub mod quick;
pub mod random;
pub mod snake;
pub mod uniform;

pub use clock::{RgbClockShow, SeparatedClockShow};
pub use data::ByteShow;
pub use demo::DemoShow;
pub use gradient::GradientShow;
pub use null::NullShow;
pub use quick::QuickShow;
pub use random::RandomShow;
pub use snake::SnakeShow;
pub use uniform::UniformShow;

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
    mut show_cancellation_token,
    mut config,
    mut remote_input,
  } = ctx.shared;
  let show_take = config.lock(|s| s.show.take());
  if let Some(mut show) = show_take {
    let mut ctrl = ColorMemoryController::new(lights, *asm_delay);

    show_cancellation_token.lock(|token| token.reset());
    Show::run(
      show.as_mut(),
      &mut show_cancellation_token,
      &mut ctrl,
      *asm_delay,
      &mut remote_input,
      &mut config,
    );
  }
  show_task::spawn().unwrap();
}

pub trait Show {
  fn run(
    &mut self,
    cancel: &mut app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut ColorMemoryController,
    asm_delay: AsmDelay,
    remote_input: &mut app::shared_resources::remote_input_lock,
    config: &mut app::shared_resources::config_lock,
  );
}

#[derive(Default)]
pub struct ShowCancellationToken(bool);

impl ShowCancellationToken {
  pub fn is_requested(&self) -> bool {
    self.0
  }

  pub fn request(&mut self) {
    self.0 = true;
  }

  fn reset(&mut self) {
    self.0 = false;
  }
}

#[macro_export]
macro_rules! return_cancel {
  ($cancellation_token_lock:ident) => {{
    if ::rtic::Mutex::lock($cancellation_token_lock, |cancellation_token| {
      cancellation_token.is_requested()
    }) {
      return;
    }
  }};
}
