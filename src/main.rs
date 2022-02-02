#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

extern crate panic_semihosting;

pub mod control;
pub mod light;

use cortex_m::delay::Delay;
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;

use alloc_cortex_m::CortexMHeap;
use alloc::{boxed::Box, vec::Vec};

use pico::{
  hal::{self, adc::Adc, clocks::ClockSource, sio::Sio, uart::UartPeripheral, watchdog::Watchdog},
  pac, PicoExplorer, XOSC_CRYSTAL_FREQ,
};

use embedded_time::fixed_point::FixedPoint;

use light::{
  color::Color,
  show::{self, Show, UniformShow},
  Lights, Utils,
};

#[link_section = ".boot_loader"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

struct App {
  lights: Lights,
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

    let (_explorer, pins) = PicoExplorer::new(
      p.IO_BANK0,
      p.PADS_BANK0,
      sio.gpio_bank0,
      p.SPI0,
      adc,
      &mut p.RESETS,
      &mut delay,
    );

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
    let _uart = UartPeripheral::new(p.UART0, &mut p.RESETS)
      .enable(
        hal::uart::common_configs::_115200_8_N_1,
        clocks.peripheral_clock.into(),
      )
      .unwrap();

    hprintln!("semihosting enabled").unwrap();
    panic!("panicking over semihosting");

    let stack: Vec<Box<dyn Show>> = vec![Box::new(UniformShow::new(Color::WHITE))];

    Self {
      lights,
      utils,

      stack,
    }
  }

  fn run(mut self) -> ! {
    loop {
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
