use super::{Lights, Utils};

pub mod demo;
pub mod uniform;
//pub mod firefly;
//pub mod gradient;
//pub mod lightning;
//pub mod off;
//pub mod pendulum;
pub mod quick;

pub use demo::DemoShow;
//pub use firefly::FireflyShow;
//pub use gradient::GradientShow;
//pub use lightning::{CollisionShow, SparkleShow};
//pub use off::OffShow;
//pub use pendulum::PendulumShow;
pub use quick::QuickShow;
pub use uniform::UniformShow;

pub enum State {
  Running,
  Finished,
}

pub trait Show {
  fn update(&mut self, lights: &mut Lights, util: &mut Utils) -> State;
}
