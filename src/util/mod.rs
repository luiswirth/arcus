pub mod uart;

#[derive(Debug, Copy, Clone)]
pub struct AsmDelay {
  cpu_freq_hz: u32,
}
impl AsmDelay {
  pub fn new(sys_freq: u32) -> Self {
    Self {
      cpu_freq_hz: sys_freq,
    }
  }
}
impl embedded_hal::blocking::delay::DelayMs<u32> for AsmDelay {
  fn delay_ms(&mut self, ms: u32) {
    cortex_m::asm::delay(self.cpu_freq_hz / 1_000 * ms);
  }
}
impl embedded_hal::blocking::delay::DelayUs<u32> for AsmDelay {
  fn delay_us(&mut self, us: u32) {
    cortex_m::asm::delay(self.cpu_freq_hz / 1_000_000 * us);
  }
}
