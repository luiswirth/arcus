use crate::{app::shared_resources::cancel_lock, light::Lights, util::AsmDelay};

use super::Show;

#[derive(Default)]
pub struct NullShow;

impl Show for NullShow {
  fn run(&mut self, _: &mut Lights, _: AsmDelay, _: &mut cancel_lock) {}
}
