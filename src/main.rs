#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]
#![feature(const_fn_floating_point_arithmetic)]

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

pub mod debug;
pub mod light;

use alloc_cortex_m::CortexMHeap;
use cortex_m::delay::Delay;
use cortex_m_rt::entry;

use alloc::{boxed::Box, vec::Vec};

use embedded_time::fixed_point::FixedPoint;
use light::{show::Show, Lights, Utils};
//use panic_semihosting as _;

use pico_explorer::{
  hal::{self, adc::Adc, clocks::ClockSource, sio::Sio, uart::UartPeripheral, watchdog::Watchdog},
  pac, PicoExplorer, XOSC_CRYSTAL_FREQ,
};

use light::show;

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

    let vec = vec![0, 1, 2, 4];
    //cortex_m_semihosting::hprintln!("{:?}", vec).unwrap();

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

    let uart = UartPeripheral::enable(
      p.UART0,
      &mut p.RESETS,
      hal::uart::common_configs::_115200_8_N_1,
      clocks.peripheral_clock.into(),
    )
    .unwrap();

    let stack = vec![Box::new(show::DemoShow::default()) as Box<dyn Show>];

    Self {
      _uart: uart,
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
