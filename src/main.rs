#![no_std]
#![no_main]
#![deny(unsafe_code)]

extern crate panic_semihosting;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// dispatchers?
#[rtic::app(device = rp_pico::hal::pac, peripherals = true)]
mod app {
  use embedded_hal::digital::v2::OutputPin;
  use embedded_time::{duration::Extensions, fixed_point::FixedPoint};
  use rp_pico::hal::{
    self,
    clocks::{self, ClockSource},
    gpio, Sio,
  };
  use systick_monotonic::Systick;

  // A monotonic timer to enable scheduling in RTIC
  #[monotonic(binds = SysTick, default = true)]
  type MyMono = Systick<100>; // 100 Hz / 10 ms granularity

  const SCAN_TIME_US: u32 = 1_000_000;

  // Resources shared between tasks
  #[shared]
  struct Shared {
    timer: hal::Timer,
    alarm: hal::timer::Alarm0,
    led: gpio::Pin<gpio::pin::bank0::Gpio25, gpio::PushPullOutput>,
  }

  // Local resources to specific tasks (cannot be shared)
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

    let mut timer = hal::Timer::new(c.device.TIMER, &mut resets);
    let mut alarm = timer.alarm_0().unwrap();
    let _ = alarm.schedule(SCAN_TIME_US.microseconds());
    alarm.enable_interrupt(&mut timer);

    (
      Shared { timer, alarm, led },
      Local {},
      init::Monotonics(mono),
    )
  }

  // Background task, runs whenever no other tasks are running
  #[idle]
  fn idle(_: idle::Context) -> ! {
    loop {
      continue;
    }
  }

  // Hardware task, bound to a hardware interrupt
  #[task(
        binds = TIMER_IRQ_0,
        priority = 1,
        shared = [timer, alarm, led],
        local = [tog: bool = true],
    )]
  fn timer_irq(mut c: timer_irq::Context) {
    if *c.local.tog {
      c.shared.led.lock(|l| l.set_high().unwrap());
    } else {
      c.shared.led.lock(|l| l.set_low().unwrap());
    }
    *c.local.tog = !*c.local.tog;

    let timer = c.shared.timer;
    let alarm = c.shared.alarm;
    (timer, alarm).lock(|t, a| {
      a.clear_interrupt(t);
      let _ = a.schedule(SCAN_TIME_US.microseconds());
    });
  }
}
