// avoid extra module indirection
pub use inner_app::*;

#[rtic::app(
    device = rp_pico::pac,
    peripherals = true,
    dispatchers = [TIMER_IRQ_1, TIMER_IRQ_2, TIMER_IRQ_3])
]
mod inner_app {
  use embedded_hal::digital::v2::OutputPin;

  use rp2040_monotonic::Rp2040Monotonic;
  use rp_pico::hal::{self, clocks, gpio, Sio};

  use crate::{
    config::Config,
    input::InputTask,
    remote::{RemoteInput, RemoteTask},
    show::{self, ShowCancellationToken},
    uprintln,
    util::uart::init_uart,
    ALLOCATOR,
  };

  pub type Monotonic = Rp2040Monotonic;

  // A monotonic timer to enable scheduling in RTIC
  // NOTE: For some reason this type can't be made public. Therefore we introduce an indirection.
  #[monotonic(binds = TIMER_IRQ_0, default = true)]
  type RticMonotonicSpecification = Monotonic;

  type LedPin = gpio::Pin<gpio::bank0::Gpio25, gpio::Output<gpio::PushPull>>;

  #[shared]
  struct Shared {
    config: Config,
    remote_input: RemoteInput,
    show_cancellation_token: ShowCancellationToken,
  }

  #[local]
  struct Local {
    show_task: show::ShowTask,
    input_task: InputTask,
    remote_task: RemoteTask,
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

    let sio = Sio::new(ctx.device.SIO);
    let pins = rp_pico::Pins::new(
      ctx.device.IO_BANK0,
      ctx.device.PADS_BANK0,
      sio.gpio_bank0,
      &mut ctx.device.RESETS,
    );

    init_uart(
      ctx.device.UART0,
      &mut ctx.device.RESETS,
      pins.gpio0.into_mode(),
      pins.gpio1.into_mode(),
      clocks.peripheral_clock.into(),
    );
    uprintln!("uart initialized.");

    let mut led: LedPin = pins.led.into_push_pull_output();
    led.set_high().unwrap();

    let config = Config::default();

    let show_task = show::ShowTask::init(
      pins.gpio2.into_mode(),
      ctx.device.PIO0,
      &clocks.system_clock,
      &mut ctx.device.RESETS,
    );
    let show_cancellation_token = ShowCancellationToken::default();

    let input_task = InputTask::default();

    let remote_input = RemoteInput::default();
    let remote_task = RemoteTask::init(pins.gpio3.into_floating_input());

    let mono = Monotonic::new(ctx.device.TIMER);

    (
      Shared {
        config,
        remote_input,
        show_cancellation_token,
      },
      Local {
        show_task,
        input_task,
        remote_task,
      },
      init::Monotonics(mono),
    )
  }

  use crate::{input::input_task, remote::remote_task, show::show_task};
  extern "Rust" {
    #[task(
        priority = 1,
        shared = [show_cancellation_token, config, remote_input],
        local = [show_task],
    )]
    fn show_task(ctx: show_task::Context);

    #[task(
        priority = 2,
        shared = [remote_input, config, show_cancellation_token],
        local = [input_task],
    )]
    fn input_task(ctx: input_task::Context);

    #[task(
        binds = IO_IRQ_BANK0,
        priority = 3,
        shared = [remote_input],
        local = [remote_task],
    )]
    fn remote_task(ctx: remote_task::Context);
  }
}
