#![no_std]
#![no_main]

mod debug;
mod lights;

use cortex_m::delay::Delay;
use cortex_m_rt::entry;

use embedded_time::fixed_point::FixedPoint;
use lights::Lights;

use pico_explorer::{
  hal::{self, adc::Adc, clocks::ClockSource, sio::Sio, uart::UartPeripheral, watchdog::Watchdog},
  pac, PicoExplorer, XOSC_CRYSTAL_FREQ,
};

#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER;

//static SHARED: Mutex<RefCell<Option<Shared>>> = Mutex::default();
//pub struct Shared {
//  pub delay: Delay,
//}

struct App {
  //p: pac::Peripherals,
  //cp: pac::CorePeripherals,
  //clocks: ClocksManager,
  //adc: Adc,
  //sio: Sio,
  //pins: Pins,
  _explorer: PicoExplorer,
  lights: Lights,
  _uart: UartPeripheral<hal::uart::Enabled, pac::UART0>,
}

impl App {
  fn init() -> Self {
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

    let lights = Lights::init(
      p.PIO0,
      pins.gpio2.into_mode(),
      &mut p.RESETS,
      clocks.system_clock.get_freq().0 as f32,
      delay,
    );

    let _uart_tx_pin = pins.gpio0.into_mode::<hal::gpio::FunctionUart>();
    let _uart_rx_pin = pins.gpio1.into_mode::<hal::gpio::FunctionUart>();

    let uart = UartPeripheral::enable(
      p.UART0,
      &mut p.RESETS,
      hal::uart::common_configs::_115200_8_N_1,
      clocks.peripheral_clock.into(),
    )
    .unwrap();

    //let shared = Shared { delay };
    //interrupt::free(|cs| SHARED.borrow(cs).replace(Some(shared)));

    Self {
      //p,
      //cp,
      //clocks,
      //adc,
      //sio,
      _explorer: explorer,
      //pins,
      _uart: uart,
      lights,
    }
  }

  /// Get a reference to the app's lights.
  fn lights_mut(&mut self) -> &mut Lights {
    &mut self.lights
  }
}

#[entry]
fn main() -> ! {
  let mut app = App::init();

  app
    .lights_mut()
    .play_lightshow(&mut lights::ExampleLightshow {});

  loop {
    cortex_m::asm::nop();
  }
}
