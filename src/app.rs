// avoid extra module indirection
pub use inner_app::*;

#[rtic::app(
    device = rp_pico::pac,
    peripherals = true,
    dispatchers = [TIMER_IRQ_0, TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3])
]
mod inner_app {
  use alloc::boxed::Box;
  use embedded_hal::digital::v2::OutputPin;
  use embedded_time::fixed_point::FixedPoint;

  use rp_pico::hal::{
    self,
    clocks::{self, ClockSource},
    gpio,
    uart::UartPeripheral,
    Sio,
  };
  use systick_monotonic::*;

  use crate::{
    light::color::Color,
    remote,
    show::{self, CancellationToken, Show},
    ALLOCATOR,
  };

  // A monotonic timer to enable scheduling in RTIC
  #[monotonic(binds = SysTick, default = true)]
  type MyMono = Systick<100_000>; // frequency in Hz determining granularity

  type LedPin = gpio::Pin<gpio::bank0::Gpio25, gpio::Output<gpio::PushPull>>;

  #[shared]
  struct Shared {
    show: Option<Box<dyn Show + Send>>,
    cancel: CancellationToken,
  }

  #[local]
  struct Local {
    remote_task: remote::RemoteTask,
    show_task: show::ShowTask,
  }

  #[init]
  fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
    let heap_start = cortex_m_rt::heap_start() as usize;
    let heap_size = 200 * 1024;
    unsafe { ALLOCATOR.init(heap_start, heap_size) }

    let mut watchdog = hal::Watchdog::new(ctx.device.WATCHDOG);
    let clocks = clocks::init_clocks_and_plls(
      rp_pico::XOSC_CRYSTAL_FREQ,
      ctx.device.XOSC,
      ctx.device.CLOCKS,
      ctx.device.PLL_SYS,
      ctx.device.PLL_USB,
      &mut ctx.device.RESETS,
      &mut watchdog,
    )
    .ok()
    .unwrap();

    let systick = ctx.core.SYST;
    let systick_freq = clocks.system_clock.get_freq().integer();
    let mono = Systick::new(systick, systick_freq);

    let sio = Sio::new(ctx.device.SIO);
    let pins = rp_pico::Pins::new(
      ctx.device.IO_BANK0,
      ctx.device.PADS_BANK0,
      sio.gpio_bank0,
      &mut ctx.device.RESETS,
    );

    let mut led: LedPin = pins.led.into_push_pull_output();
    led.set_low().unwrap();

    let _uart_tx_pin = pins.gpio0.into_mode::<hal::gpio::FunctionUart>();
    let _uart_rx_pin = pins.gpio1.into_mode::<hal::gpio::FunctionUart>();
    let _uart = UartPeripheral::new(ctx.device.UART0, &mut ctx.device.RESETS)
      .enable(
        hal::uart::common_configs::_115200_8_N_1,
        clocks.peripheral_clock.into(),
      )
      .unwrap();

    let show: Option<Box<dyn Show + Send>> = Some(Box::new(show::UniformShow::new(Color::WHITE)));
    let cancel = CancellationToken::default();

    let show_task = show::ShowTask::init(
      pins.gpio2.into_mode(),
      ctx.device.PIO0,
      &clocks.system_clock,
      &mut ctx.device.RESETS,
    );

    let remote_task = remote::RemoteTask::init(
      pins.gpio3.into_floating_input(),
      ctx.device.TIMER,
      &mut ctx.device.RESETS,
    );

    (
      Shared { show, cancel },
      Local {
        show_task,
        remote_task,
      },
      init::Monotonics(mono),
    )
  }

  use crate::{remote::remote_task, show::show_task};
  extern "Rust" {
    #[task(
        priority = 1,
        shared = [show, cancel],
        local = [show_task],
    )]
    fn show_task(ctx: show_task::Context);

    #[task(
        binds = IO_IRQ_BANK0,
        priority = 2,
        shared = [show, cancel],
        local = [remote_task],
    )]
    fn remote_task(ctx: remote_task::Context);
  }
}
