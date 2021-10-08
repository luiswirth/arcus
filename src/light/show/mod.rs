use super::{Lights, Utils};

pub mod demo;
pub mod firefly;
pub mod gradient;
pub mod lightning;
pub mod pendulum;
pub mod quick;

pub trait Show {
  fn play(&mut self, lights: &mut Lights, util: &mut Utils);
}
