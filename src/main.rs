#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

pub mod control;
pub mod debug;
pub mod light;

use alloc_cortex_m::CortexMHeap;
use cortex_m::delay::Delay;
use cortex_m_rt::entry;

use alloc::{boxed::Box, vec::Vec};

use embedded_time::fixed_point::FixedPoint;
use light::{show::Show, Lights, Utils};
//use panic_semihosting as _;

use pico::{
  hal::{self, adc::Adc, clocks::ClockSource, sio::Sio, uart::UartPeripheral, watchdog::Watchdog},
  pac, PicoExplorer, XOSC_CRYSTAL_FREQ,
};

use control::IrRemote;
use light::show;

use self::{
  control::RemoteKey,
  light::{color::Color, show::UniformShow},
};

#[link_section = ".boot_loader"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

/// Show Managment
///
/// - Light show stack
///     this enables switching to informational shows or animations
///     and then returning back to the original show.
///

struct App {
  lights: Lights,
  remote: IrRemote,
  _uart: UartPeripheral<hal::uart::Enabled, pac::UART0>,
  utils: Utils,

  stack: Vec<Box<dyn Show>>,
}

impl App {
  fn init() -> Self {
    // Setup global allocator with conservative heap.
    {
      let start = cortex_m_rt::heap_start() as usize;
      let size = 200 * 1024;
      unsafe { ALLOCATOR.init(start, size) }
    }

    let mut p = pac::Peripherals::take().unwrap();
    let cp = pac::CorePeripherals::take().unwrap();

    let mut watchdog = Watchdog::new(p.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
      XOSC_CRYSTAL_FREQ,
      p.XOSC,
      p.CLOCKS,
      p.PLL_SYS,
      p.PLL_USB,
      &mut p.RESETS,
      &mut watchdog,
    )
    .ok()
    .unwrap();
    let mut delay = Delay::new(cp.SYST, clocks.system_clock.get_freq().integer());

    let adc = Adc::new(p.ADC, &mut p.RESETS);
    let sio = Sio::new(p.SIO);

    let (explorer, pins) = PicoExplorer::new(
      p.IO_BANK0,
      p.PADS_BANK0,
      sio.gpio_bank0,
      p.SPI0,
      adc,
      &mut p.RESETS,
      &mut delay,
    );

    debug::init_debug(explorer.screen);

    let lights = Lights::init(
      p.PIO0,
      &mut p.RESETS,
      clocks.system_clock.get_freq().0 as f32,
      pins.gpio2.into_mode(),
      p.TIMER,
    );

    let utils = Utils::new(delay);

    let _uart_tx_pin = pins.gpio0.into_mode::<hal::gpio::FunctionUart>();
    let _uart_rx_pin = pins.gpio1.into_mode::<hal::gpio::FunctionUart>();

    let uart = UartPeripheral::new(p.UART0, &mut p.RESETS)
      .enable(
        hal::uart::common_configs::_115200_8_N_1,
        clocks.peripheral_clock.into(),
      )
      .unwrap();

    let stack: Vec<Box<dyn Show>> = vec![Box::new(show::QuickShow::default())];

    let ir_pin = pins.gpio3.into_pull_down_input();
    let remote = IrRemote::new(ir_pin);

    Self {
      _uart: uart,
      lights,
      remote,
      utils,

      stack,
    }
  }

  fn run(mut self) -> ! {
    loop {
      match self.remote.get_key() {
        Some(RemoteKey::Num0) => self.stack.push(Box::new(UniformShow::new(Color::BLUE))),
        Some(RemoteKey::Num1) => self.stack.push(Box::new(UniformShow::new(Color::WHITE))),
        Some(RemoteKey::Num2) => self.stack.push(Box::new(UniformShow::new(Color::WHITE))),
        _ => {}
      }

      if let Some(show) = self.stack.last_mut() {
        let state = Show::update(show.as_mut(), &mut self.lights, &mut self.utils);
        if matches!(state, show::State::Finished) {
          self.stack.pop();
        }
      }
    }
  }
}

#[entry]
fn main() -> ! {
  let app = App::init();
  app.run();
}
