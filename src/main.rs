#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use alloc_cortex_m::CortexMHeap;

pub mod light;

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

extern crate panic_semihosting;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// TODO: choose the right dispatchers
#[rtic::app(
    device = rp_pico::hal::pac,
    peripherals = true,
    dispatchers = [TIMER_IRQ_0, TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3])
]
mod app {
  use alloc::boxed::Box;
  use embedded_hal::digital::v2::OutputPin;
  use embedded_time::fixed_point::FixedPoint;

  use rp_pico::{
    hal::{
      self,
      clocks::{self, ClockSource},
      gpio,
      uart::{self, UartPeripheral},
      Sio,
    },
    pac,
  };
  use systick_monotonic::*;

  use crate::{
    light::{
      color::Color,
      show::{self, Show, UniformShow},
      Lights,
    },
    ALLOCATOR,
  };

  // A monotonic timer to enable scheduling in RTIC
  #[monotonic(binds = SysTick, default = true)]
  type MyMono = Systick<100>; // 100 Hz / 10 ms granularity

  // Resources shared between tasks.
  #[shared]
  struct Shared {
    led: gpio::Pin<gpio::pin::bank0::Gpio25, gpio::PushPullOutput>,
    _uart: UartPeripheral<uart::Enabled, pac::UART0>,
  }

  // Local resources to specific tasks (cannot be shared).
  #[local]
  struct Local {
    lights: Lights,
    show: Option<Box<dyn Show + Send>>,
  }

  #[init]
  fn init(mut c: init::Context) -> (Shared, Local, init::Monotonics) {
    {
      let start = cortex_m_rt::heap_start() as usize;
      let size = 200 * 1024;
      unsafe { ALLOCATOR.init(start, size) }
    }

    let mut watchdog = hal::Watchdog::new(c.device.WATCHDOG);
    let clocks = clocks::init_clocks_and_plls(
      rp_pico::XOSC_CRYSTAL_FREQ,
      c.device.XOSC,
      c.device.CLOCKS,
      c.device.PLL_SYS,
      c.device.PLL_USB,
      &mut c.device.RESETS,
      &mut watchdog,
    )
    .ok()
    .unwrap();

    let systick = c.core.SYST;
    let systick_freq = clocks.system_clock.get_freq().integer();
    let mono = Systick::new(systick, systick_freq);

    let sio = Sio::new(c.device.SIO);
    let pins = rp_pico::Pins::new(
      c.device.IO_BANK0,
      c.device.PADS_BANK0,
      sio.gpio_bank0,
      &mut c.device.RESETS,
    );
    let mut led = pins.led.into_push_pull_output();
    led.set_low().unwrap();

    let _uart_tx_pin = pins.gpio0.into_mode::<hal::gpio::FunctionUart>();
    let _uart_rx_pin = pins.gpio1.into_mode::<hal::gpio::FunctionUart>();
    let uart = UartPeripheral::new(c.device.UART0, &mut c.device.RESETS)
      .enable(
        hal::uart::common_configs::_115200_8_N_1,
        clocks.peripheral_clock.into(),
      )
      .unwrap();

    let lights = Lights::init(
      c.device.PIO0,
      &mut c.device.RESETS,
      clocks.system_clock.get_freq().0 as f32,
      pins.gpio2.into_mode(),
    );

    //let show = None;
    let show: Option<Box<dyn Show + Send>> = Some(Box::new(UniformShow::new(Color::WHITE)));

    led_blink::spawn().unwrap();
    light_task::spawn().unwrap();

    (
      Shared { led, _uart: uart },
      Local { lights, show },
      init::Monotonics(mono),
    )
  }

  // Background task, runs whenever no other tasks are running.
  #[idle]
  fn idle(_: idle::Context) -> ! {
    loop {
      continue;
    }
  }

  #[task(
        priority = 1,
        shared = [],
        local = [lights, show],
    )]
  fn light_task(c: light_task::Context) {
    if let Some(show) = c.local.show {
      let state = Show::update(show.as_mut(), &mut *c.local.lights);
      if matches!(state, show::State::Finished) {
        *c.local.show = None;
      }
    }

    light_task::spawn().unwrap();
  }

  #[task(
        priority = 1,
        shared = [led],
        local = [tog: bool = true],
    )]
  fn led_blink(mut c: led_blink::Context) {
    if *c.local.tog {
      c.shared.led.lock(|l| l.set_high().unwrap());
    } else {
      c.shared.led.lock(|l| l.set_low().unwrap());
    }
    *c.local.tog = !*c.local.tog;

    led_blink::spawn_after(1.secs()).unwrap();
  }
}
