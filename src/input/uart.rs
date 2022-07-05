use crate::{
  app::{
    input_task,
    uart_task::{self, SharedResources},
  },
  util::uart::UART_PERIPHERAL,
};
use rtic::Mutex;

pub struct UartTask {}
impl UartTask {
  pub fn init() -> Self {
    Self {}
  }
}

pub fn uart_task(ctx: uart_task::Context) {
  let UartTask {} = ctx.local.uart_task;
  let SharedResources { mut remote_input } = ctx.shared;

  let mut data = [0u8; 1024];
  let mut string: &str = "";
  let mut nbytes = 0;

  cortex_m::interrupt::free(|cs| {
    let uart = UART_PERIPHERAL.borrow(cs);
    let mut uart = uart.borrow_mut();
    let uart = uart.as_mut().expect("uart not initialized");

    use core::fmt::Write;
    match uart.read_raw(&mut data) {
      Ok(n) => nbytes = n,
      Err(_e) => {
        let _ = writeln!(uart, "uart read error");
      }
    }
    match core::str::from_utf8(&data[0..nbytes]) {
      Ok(s) => string = s,
      Err(e) => {
        let _ = writeln!(uart, "uart utf8 error: {}", e);
      }
    }
    let _ = writeln!(uart, "UART echo: {}", string);
  });

  use infrared::remotecontrol as irrc;
  let action = match string {
    "1" => irrc::Action::One,
    "2" => irrc::Action::Two,
    "3" => irrc::Action::Three,
    "4" => irrc::Action::Four,
    "5" => irrc::Action::Five,
    "6" => irrc::Action::Six,
    "7" => irrc::Action::Seven,
    "8" => irrc::Action::Eight,
    "9" => irrc::Action::Nine,
    "0" => irrc::Action::Zero,
    "s" => irrc::Action::Stop,
    "p" => irrc::Action::Play_Pause,
    "t" => irrc::Action::Time,
    "x" => irrc::Action::Teletext,
    "r" => irrc::Action::Repeat,
    "?" => irrc::Action::Random,
    "g" => irrc::Action::Prog,
    "<" => irrc::Action::Prev,
    ">" => irrc::Action::Next,
    "[" => irrc::Action::Rewind,
    "]" => irrc::Action::Forward,
    _ => return,
  };
  remote_input.lock(|input| {
    input.0 = Some(action);
  });
  input_task::spawn().unwrap();
}
