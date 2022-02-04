#![no_std]
#![no_main]
#![feature(default_alloc_error_handler)]

use alloc_cortex_m::CortexMHeap;

pub mod light;
pub mod show;

#[allow(unused_imports)]
#[macro_use]
extern crate alloc;

extern crate panic_semihosting;

#[link_section = ".boot_loader"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[rtic::app(
    device = rp_pico::hal::pac,
    peripherals = true,
    dispatchers = [TIMER_IRQ_0, TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3])
]
mod app {
  use alloc::{boxed::Box, string::ToString};
  use embedded_hal::digital::v2::OutputPin;
  use embedded_time::fixed_point::FixedPoint;

  use rp_pico::{
    hal::{
      self,
      clocks::{self, ClockSource},
      gpio,
      uart::{self, UartPeripheral},
      Sio, Timer,
    },
    pac,
  };
  use systick_monotonic::*;

  use crate::{
    light::{color::Color, Lights},
    show::{self, Show},
    ALLOCATOR,
  };

  // A monotonic timer to enable scheduling in RTIC
  #[monotonic(binds = SysTick, default = true)]
  type MyMono = Systick<100_000>; // frequency in Hz determining granularity

  type IrReceiver = infrared::Receiver<
    infrared::protocol::Nec,
    infrared::receiver::Poll,
    infrared::receiver::PinInput<gpio::Pin<gpio::bank0::Gpio3, gpio::Input<gpio::Floating>>>,
    //Button<CarMp3>,
  >;

  type LedPin = gpio::Pin<gpio::bank0::Gpio25, gpio::Output<gpio::PushPull>>;
  type LightsPin = gpio::Pin<gpio::bank0::Gpio2, gpio::FunctionPio0>;
  type IrReceiverPin = gpio::Pin<gpio::bank0::Gpio3, gpio::Input<gpio::Floating>>;

  const RECEIVER_FREQ_HZ: u32 = 20_000;
  const RECEIVER_DURATION_US: u32 = 1_000_000 / RECEIVER_FREQ_HZ;

  // Resources shared between tasks.
  #[shared]
  struct Shared {
    show: Option<Box<dyn Show + Send>>,
    _led: gpio::Pin<gpio::pin::bank0::Gpio25, gpio::PushPullOutput>,
    uart: UartPeripheral<uart::Enabled, pac::UART0>,
  }

  // Local resources to specific tasks (cannot be shared).
  #[local]
  struct Local {
    lights: Lights,
    ir_receiver: IrReceiver,
    timer: Timer,
  }

  #[init]
  fn init(mut c: init::Context) -> (Shared, Local, init::Monotonics) {
    let heap_start = cortex_m_rt::heap_start() as usize;
    let heap_size = 200 * 1024;
    unsafe { ALLOCATOR.init(heap_start, heap_size) }

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
    let timer = Timer::new(c.device.TIMER, &mut c.device.RESETS);

    let systick = c.core.SYST;
    // TODO: is this the right systick frequency?
    let systick_freq = clocks.system_clock.get_freq().integer();
    let mono = Systick::new(systick, systick_freq);

    let sio = Sio::new(c.device.SIO);
    let pins = rp_pico::Pins::new(
      c.device.IO_BANK0,
      c.device.PADS_BANK0,
      sio.gpio_bank0,
      &mut c.device.RESETS,
    );

    let mut led: LedPin = pins.led.into_push_pull_output();
    led.set_low().unwrap();

    let _uart_tx_pin = pins.gpio0.into_mode::<hal::gpio::FunctionUart>();
    let _uart_rx_pin = pins.gpio1.into_mode::<hal::gpio::FunctionUart>();
    let uart = UartPeripheral::new(c.device.UART0, &mut c.device.RESETS)
      .enable(
        hal::uart::common_configs::_115200_8_N_1,
        clocks.peripheral_clock.into(),
      )
      .unwrap();
    uart.write_full_blocking(systick_freq.to_string().as_bytes());

    let lights_pin: LightsPin = pins.gpio2.into_mode();
    let lights = Lights::init(
      c.device.PIO0,
      &mut c.device.RESETS,
      clocks.system_clock.get_freq().0 as f32,
      lights_pin,
    );
    light_task::spawn().unwrap();
    let show: Option<Box<dyn Show + Send>> = Some(Box::new(show::UniformShow::new(Color::WHITE)));

    let ir_pin: IrReceiverPin = pins.gpio3.into_floating_input();
    let ir_receiver: IrReceiver = infrared::Receiver::builder()
      .nec()
      .polled()
      .resolution(RECEIVER_FREQ_HZ)
      .pin(ir_pin)
      .build();
    ir_remote_task::spawn().unwrap();
    uart.write_full_blocking(format!("ranges: ({:?})", ir_receiver.ranges()).as_bytes());

    (
      Shared {
        _led: led,
        show,
        uart,
      },
      Local {
        lights,
        ir_receiver,
        timer,
      },
      init::Monotonics(mono),
    )
  }

  #[task(
        priority = 1,
        shared = [show],
        local = [lights, timer],
    )]
  fn light_task(mut c: light_task::Context) {
    c.shared.show.lock(|show_option| {
      if let Some(show) = show_option {
        let state = Show::update(
          show.as_mut(),
          &mut *c.local.lights,
          c.local.timer.count_down(),
        );
        if matches!(state, show::State::Finished) {
          *show_option = None;
        }
      }
    });

    light_task::spawn().unwrap();
  }

  #[task(
        priority = 2,
        shared = [show, uart],
        local = [ir_receiver, color_idx: u32 = 0],
    )]
  fn ir_remote_task(mut c: ir_remote_task::Context) {
    ir_remote_task::spawn_after((RECEIVER_DURATION_US as u64).micros()).unwrap();

    let cmd = c.local.ir_receiver.poll();
    let string = match cmd {
      Err(e) => Some(format!("Error: {:?} ", e)),
      Ok(Some(e)) => Some(format!("{:?} ", e)),
      Ok(None) => None,
    };
    if let Some(string) = string {
      c.shared
        .uart
        .lock(|uart| uart.write_full_blocking(string.as_bytes()));
    }

    if let Ok(Some(_)) = cmd {
      let color_idx = c.local.color_idx;
      c.shared
        .uart
        .lock(|uart| uart.write_full_blocking(color_idx.to_string().as_bytes()));
      let new_show: Box<dyn Show + Send> = match *color_idx % 3 {
        0 => Box::new(show::UniformShow::new(Color::RED)),
        1 => Box::new(show::UniformShow::new(Color::GREEN)),
        2 => Box::new(show::UniformShow::new(Color::BLUE)),
        _ => Box::new(show::UniformShow::new(Color::NONE)),
      };
      c.shared.show.lock(|show| *show = Some(new_show));
      *color_idx = color_idx.wrapping_add(1);
    }
  }
}
