use crate::light::Lights;

use rp_pico::hal::timer::CountDown;

pub mod demo;
pub mod quick;
pub mod uniform;

pub use demo::DemoShow;
pub use quick::QuickShow;
pub use uniform::UniformShow;

pub enum State {
  Running,
  Finished,
}

pub trait Show {
  fn update(&mut self, lights: &mut Lights, count_down: CountDown) -> State;
}
