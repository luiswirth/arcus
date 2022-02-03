#![no_std]
#![no_main]
#![deny(unsafe_code)]

extern crate panic_semihosting;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// TODO: choose the right dispatchers
#[rtic::app(device = rp_pico::hal::pac, peripherals = true, dispatchers = [SPI0_IRQ, SPI1_IRQ])]
mod app {
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

  // A monotonic timer to enable scheduling in RTIC
  #[monotonic(binds = SysTick, default = true)]
  type MyMono = Systick<100>; // 100 Hz / 10 ms granularity

  // Resources shared between tasks.
  #[shared]
  struct Shared {
    led: gpio::Pin<gpio::pin::bank0::Gpio25, gpio::PushPullOutput>,
    uart: UartPeripheral<uart::Enabled, pac::UART0>,
  }

  // Local resources to specific tasks (cannot be shared).
  #[local]
  struct Local {}

  #[init]
  fn init(c: init::Context) -> (Shared, Local, init::Monotonics) {
    let mut resets = c.device.RESETS;
    let mut watchdog = hal::Watchdog::new(c.device.WATCHDOG);
    let clocks = clocks::init_clocks_and_plls(
      rp_pico::XOSC_CRYSTAL_FREQ,
      c.device.XOSC,
      c.device.CLOCKS,
      c.device.PLL_SYS,
      c.device.PLL_USB,
      &mut resets,
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
      &mut resets,
    );
    let mut led = pins.led.into_push_pull_output();
    led.set_low().unwrap();

    let _uart_tx_pin = pins.gpio0.into_mode::<hal::gpio::FunctionUart>();
    let _uart_rx_pin = pins.gpio1.into_mode::<hal::gpio::FunctionUart>();
    let uart = UartPeripheral::new(c.device.UART0, &mut resets)
      .enable(
        hal::uart::common_configs::_115200_8_N_1,
        clocks.peripheral_clock.into(),
      )
      .unwrap();

    led_blink::spawn().unwrap();
    uart_hello::spawn().unwrap();

    (Shared { led, uart }, Local {}, init::Monotonics(mono))
  }

  // Background task, runs whenever no other tasks are running.
  #[idle]
  fn idle(_: idle::Context) -> ! {
    loop {
      continue;
    }
  }

  // Software task, not bound to a hardware interrupt.
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

  // Software task, not bound to a hardware interrupt.
  #[task(
        priority = 2,
        shared = [uart],
    )]
  fn uart_hello(mut c: uart_hello::Context) {
    c.shared
      .uart
      .lock(|u| u.write_full_blocking(b"hello from uart\n"));

    uart_hello::spawn_after(1.secs()).unwrap();
  }
}
