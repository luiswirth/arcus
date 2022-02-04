use crate::{
  app::remote_task,
  light::color::Color,
  show::{self, Show},
};

use alloc::{boxed::Box, string::ToString};
use rp_pico::hal::gpio;
use rtic::Mutex;
use systick_monotonic::*;

pub type IrReceiverPin = gpio::Pin<gpio::bank0::Gpio3, gpio::Input<gpio::Floating>>;
pub type IrReceiver = infrared::Receiver<
  infrared::protocol::Nec,
  infrared::receiver::Poll,
  infrared::receiver::PinInput<gpio::Pin<gpio::bank0::Gpio3, gpio::Input<gpio::Floating>>>,
  //Button<CarMp3>,
>;

pub const RECEIVER_FREQ_HZ: u32 = 20_000;
pub const RECEIVER_DURATION_US: u32 = 1_000_000 / RECEIVER_FREQ_HZ;

pub struct RemoteTask {
  ir_receiver: IrReceiver,
  counter: u32,
}
impl RemoteTask {
  pub fn init(ir_pin: IrReceiverPin) -> Self {
    let ir_receiver: IrReceiver = infrared::Receiver::builder()
      .nec()
      .polled()
      .resolution(RECEIVER_FREQ_HZ)
      .pin(ir_pin)
      .build();
    remote_task::spawn().unwrap();

    Self {
      ir_receiver,
      counter: 0,
    }
  }
}

pub fn remote_task(mut ctx: remote_task::Context) {
  let this = ctx.local.remote_task;

  remote_task::spawn_after((RECEIVER_DURATION_US as u64).micros()).unwrap();

  let cmd = this.ir_receiver.poll();
  let string = match cmd {
    Err(e) => Some(format!("Error: {:?} ", e)),
    Ok(Some(e)) => Some(format!("{:?} ", e)),
    Ok(None) => None,
  };
  if let Some(string) = string {
    ctx
      .shared
      .uart
      .lock(|uart| uart.write_full_blocking(string.as_bytes()));
  }

  if let Ok(Some(_)) = cmd {
    ctx
      .shared
      .uart
      .lock(|uart| uart.write_full_blocking(this.counter.to_string().as_bytes()));
    ctx
      .shared
      .show
      .lock(|show| *show = Some(next_show(this.counter)));
    this.counter = this.counter.wrapping_add(1);
  }
}

fn next_show(counter: u32) -> Box<dyn Show + Send> {
  match counter % 6 {
    0 => Box::new(show::UniformShow::new(Color::ALL)),
    1 => Box::new(show::UniformShow::new(Color::WHITE)),
    2 => Box::new(show::UniformShow::new(Color::RED)),
    3 => Box::new(show::UniformShow::new(Color::GREEN)),
    4 => Box::new(show::UniformShow::new(Color::BLUE)),
    5 => Box::new(show::UniformShow::new(Color::NONE)),
    _ => Box::new(show::UniformShow::new(Color::NONE)),
  }
}
