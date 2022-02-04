use crate::{app::show_task, light::Lights};

use rp_pico::{
  hal::{self, clocks::ClockSource, gpio, timer::CountDown, Timer},
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
  timer: Timer,
  lights: Lights,
}

impl ShowTask {
  pub fn init(
    lights_pin: LightsPin,
    pio0: pac::PIO0,
    timer: pac::TIMER,
    resets: &mut pac::RESETS,
    clock: &hal::clocks::SystemClock,
  ) -> Self {
    let timer = Timer::new(timer, resets);
    let lights = Lights::init(pio0, resets, clock.get_freq().0 as f32, lights_pin);
    show_task::spawn().unwrap();

    Self { lights, timer }
  }
}

pub fn show_task(mut ctx: show_task::Context) {
  let this = ctx.local.show_task;
  ctx.shared.show.lock(|show_option| {
    if let Some(show) = show_option {
      let state = Show::update(show.as_mut(), &mut this.lights, this.timer.count_down());
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
