use crate::{
  app::remote_task,
  light::color::Color,
  show::{self, Show},
};

use alloc::boxed::Box;
use infrared::{self as ir, remotecontrol as irrc};
use rp_pico::hal::gpio;
use rtic::Mutex;
use systick_monotonic::*;

pub type IrReceiverPin = gpio::Pin<gpio::bank0::Gpio3, gpio::Input<gpio::Floating>>;
pub type IrReceiver = infrared::Receiver<
  ir::protocol::Nec,
  ir::receiver::Poll,
  ir::receiver::PinInput<gpio::Pin<gpio::bank0::Gpio3, gpio::Input<gpio::Floating>>>,
  irrc::Button<CarMp3>,
>;

pub const RECEIVER_FREQ_HZ: u32 = 20_000;
pub const RECEIVER_DURATION_US: u32 = 1_000_000 / RECEIVER_FREQ_HZ;

pub struct RemoteTask {
  ir_receiver: IrReceiver,
  counter: u32,
}
impl RemoteTask {
  pub fn init(ir_pin: IrReceiverPin) -> Self {
    let ir_receiver: IrReceiver = ir::Receiver::builder()
      .pin(ir_pin)
      .polled()
      .resolution(RECEIVER_FREQ_HZ)
      .nec()
      .remotecontrol(CarMp3)
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
    Err(e) => Some(format!("IrError: {:?}\n", e)),
    Ok(Some(ref cmd)) => Some(format!(
      "IrCmd: {:?} IrAction: {:?}\n",
      cmd.command(),
      cmd.action()
    )),
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

#[derive(Default, Debug)]
pub struct CarMp3;
impl irrc::RemoteControlModel for CarMp3 {
  const MODEL: &'static str = "Car Mp3";

  const DEVTYPE: irrc::DeviceType = irrc::DeviceType::Generic;

  const PROTOCOL: ir::ProtocolId = ir::ProtocolId::Nec;

  const ADDRESS: u32 = 0;

  type Cmd = ir::protocol::NecCommand;

  const BUTTONS: &'static [(u32, infrared::remotecontrol::Action)] = &[
    (0xFFA25D, irrc::Action::ChannelListPrev),
    (0xFF629D, irrc::Action::ChannelList),
    (0xFFE21D, irrc::Action::ChannelListNext),
    (0xFF22DD, irrc::Action::Prev),
    (0xFF02FD, irrc::Action::Next),
    (0xFFC23D, irrc::Action::Play_Pause),
    (0xFFE01F, irrc::Action::VolumeDown),
    (0xFFA857, irrc::Action::VolumeUp),
    (0xFF906F, irrc::Action::Eq),
    (0xFF6897, irrc::Action::Zero),
    //(0xFF9867, irrc::Action::?),
    //(0xFFB04F, irrc::Action::?),
    (0xFF30CF, irrc::Action::One),
    (0xFF18E7, irrc::Action::Two),
    (0xFF7A85, irrc::Action::Three),
    (0xFF10EF, irrc::Action::Four),
    (0xFF38C7, irrc::Action::Five),
    (0xFF5AA5, irrc::Action::Six),
    (0xFF42BD, irrc::Action::Seven),
    (0xFF4AB5, irrc::Action::Eight),
    (0xFF52AD, irrc::Action::Nine),
  ];
}
