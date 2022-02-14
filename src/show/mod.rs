use crate::{
  app::show_task::{self, SharedResources},
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
  let SharedResources { mut show } = ctx.shared;
  let show_take = show.lock(|show| show.take());
  if let Some(mut show_take) = show_take {
    let state = Show::update(show_take.as_mut(), lights, *asm_delay);
    if matches!(state, State::Running) {
      show.lock(|show| {
        if show.is_none() {
          *show = Some(show_take);
        }
      });
    }
  }

  show_task::spawn().unwrap();
}

pub enum State {
  Running,
  Finished,
}

pub trait Show {
  fn update(&mut self, lights: &mut Lights, asm_delay: AsmDelay) -> State;
}
