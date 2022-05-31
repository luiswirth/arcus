use super::Show;

#[derive(Default)]
pub struct NullShow;

impl Show for NullShow {
  fn run(
    &mut self,
    _cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    _ctrl: &mut crate::light::controller::U32MemoryController,
    _asm_delay: crate::util::AsmDelay,
    _remote_input: &mut crate::app::shared_resources::remote_input_lock,
    _configuration: &mut crate::app::shared_resources::configuration_lock,
  ) {
    // do nothing
  }
}
