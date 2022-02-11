use crate::{
  app::show_task::{self, SharedResources},
  light::Lights,
};

use rp_pico::{
  hal::{self, clocks::ClockSource, gpio, timer::CountDown},
  pac,
};
use rtic::mutex_prelude::TupleExt02;

pub mod demo;
pub mod quick;
pub mod uniform;

pub use demo::DemoShow;
pub use quick::QuickShow;
pub use uniform::UniformShow;

pub type LightsPin = gpio::Pin<gpio::bank0::Gpio2, gpio::FunctionPio0>;

pub struct ShowTask {
  lights: Lights,
}

impl ShowTask {
  pub fn init(
    lights_pin: LightsPin,
    pio0: pac::PIO0,
    clock: &hal::clocks::SystemClock,
    resets: &mut pac::RESETS,
  ) -> Self {
    let lights = Lights::init(pio0, resets, clock.get_freq().0 as f32, lights_pin);
    show_task::spawn().unwrap();

    Self { lights }
  }
}

pub fn show_task(ctx: show_task::Context) {
  let ShowTask { lights } = ctx.local.show_task;
  let SharedResources { show, timer } = ctx.shared;
  (show, timer).lock(|show_option, timer| {
    if let Some(show) = show_option {
      let state = Show::update(show.as_mut(), lights, timer.count_down());
      if matches!(state, State::Finished) {
        *show_option = None;
      }
    }
  });

  show_task::spawn().unwrap();
}

pub enum State {
  Running,
  Finished,
}

pub trait Show {
  fn update(&mut self, lights: &mut Lights, count_down: CountDown) -> State;
}
