<<<<<<< HEAD
use crate::{
  app::{
    self, monotonics,
    remote_task::{self, SharedResources},
  },
  light::color::NormColor,
  show::{self, Show},
=======
use crate::app::{
  input_task,
  remote_task::{self, SharedResources},
>>>>>>> input-handler
};

use infrared::{self as ir, remotecontrol as irrc};
<<<<<<< HEAD
use rp_pico::hal::gpio;
use rtic::mutex_prelude::TupleExt02;
=======
use rp_pico::{
  hal::{gpio, Timer},
  pac,
};
use rtic::Mutex;

#[derive(Debug, Default)]
pub struct RemoteInput(pub Option<irrc::Action>);
>>>>>>> input-handler

type IrProto = infrared::protocol::Nec;
type IrCommand = <IrProto as infrared::Protocol>::Cmd;
pub type IrReceiverPin = gpio::Pin<gpio::bank0::Gpio3, gpio::Input<gpio::Floating>>;
pub type IrReceiver = infrared::Receiver<
  IrProto,
  IrReceiverPin,
  <app::Monotonic as rtic::Monotonic>::Instant,
  irrc::Button<NadRc512>,
>;

pub struct RemoteTask {
  ir_receiver: IrReceiver,
}
impl RemoteTask {
  pub fn init(ir_pin: IrReceiverPin) -> Self {
    ir_pin.set_interrupt_enabled(gpio::Interrupt::EdgeHigh, true);
    ir_pin.set_interrupt_enabled(gpio::Interrupt::EdgeLow, true);

    let ir_receiver = ir::Receiver::with_fugit64(ir_pin);
    Self { ir_receiver }
  }
}

pub fn remote_task(ctx: remote_task::Context) {
<<<<<<< HEAD
  let RemoteTask { ir_receiver } = ctx.local.remote_task;
  let SharedResources { show, cancel } = ctx.shared;
=======
  let RemoteTask {
    ir_receiver,
    timer,
    last_event_instant,
  } = ctx.local.remote_task;
  let SharedResources { mut remote_input } = ctx.shared;
>>>>>>> input-handler

  let now = monotonics::now();
  match ir_receiver.event_instant(now) {
    Ok(Some(cmd)) => match cmd.action() {
      Some(action) => {
        remote_input.lock(|input| {
          input.0 = Some(action);
        });
      }
      None => {}
    },
    // TODO: handle
    Err(_e) => {}
    Ok(None) => {}
  };
<<<<<<< HEAD

  let pin = ir_receiver.pin_mut();
  pin.clear_interrupt(gpio::Interrupt::EdgeHigh);
  pin.clear_interrupt(gpio::Interrupt::EdgeLow);
}

#[rustfmt::skip]
fn number_from_action(action: irrc::Action) -> Option<usize> {
  match action {
    irrc::Action::Zero  => Some(0),
    irrc::Action::One   => Some(1),
    irrc::Action::Two   => Some(2),
    irrc::Action::Three => Some(3),
    irrc::Action::Four  => Some(4),
    irrc::Action::Five  => Some(5),
    irrc::Action::Six   => Some(6),
    irrc::Action::Seven => Some(7),
    irrc::Action::Eight => Some(8),
    irrc::Action::Nine  => Some(9),
    _ => None,
  }
}

fn color_from_action(action: irrc::Action) -> Option<NormColor> {
  number_from_action(action).map(|i| {
    if i == 0 {
      NormColor::NONE
    } else {
      NormColor::STANDARD_PALETTE[i - 1]
    }
  })
}

#[rustfmt::skip]
fn next_show(action: irrc::Action) -> Option<Box<dyn Show + Send>> {
  macro_rules! show {
    ($s:expr) => {
      Some(Box::new($s))
    };
  }

  const BYTES: &[u8] = &[0b1010_1010, 0b1111_1111, 0b0000_0000, 0b1100_1100];

  if let Some(color) = color_from_action(action) {
    show!(show::UniformShow::new(color))
  } else {
    match action {
      irrc::Action::Stop       => show!(show::NullShow::default()),
      irrc::Action::Play_Pause => show!(show::SnakeShow::default()),
      irrc::Action::Random     => show!(show::RandomShow::default()),
      irrc::Action::Time       => show!(show::SeparatedClockShow::default()),
      irrc::Action::Repeat     => show!(show::ByteShow::new(BYTES)),
      //irrc::Action::?        => None,
      //irrc::Action::Prog       => show!(show::DemoShow::default()),
      irrc::Action::Prev       => None,
      irrc::Action::Next       => None,
      irrc::Action::Rewind     => None,
      irrc::Action::Forward    => None,
      _ => None,
    }
  }
=======
  input_task::spawn().unwrap();
>>>>>>> input-handler
}

#[derive(Default, Debug)]
pub struct NadRc512;
impl irrc::RemoteControlModel for NadRc512 {
  const MODEL: &'static str = "NAD RC512";

  const DEVTYPE: irrc::DeviceType = irrc::DeviceType::Generic;

  const PROTOCOL: ir::ProtocolId = ir::ProtocolId::Nec;

  const ADDRESS: u32 = 135;

  type Cmd = IrCommand;

  const BUTTONS: &'static [(u32, irrc::Action)] = &[
    (2, irrc::Action::Stop),
    (1, irrc::Action::Play_Pause),
    (3, irrc::Action::Random),
    (8, irrc::Action::Time),
    (10, irrc::Action::Repeat),
    (12, irrc::Action::One),
    (13, irrc::Action::Two),
    (14, irrc::Action::Three),
    (15, irrc::Action::Four),
    (16, irrc::Action::Five),
    (17, irrc::Action::Six),
    (18, irrc::Action::Seven),
    (19, irrc::Action::Eight),
    (21, irrc::Action::Nine),
    // TODO: map missing action
    //(77, irrc::Action::?),
    (76, irrc::Action::Zero),
    (11, irrc::Action::Prog),
    (5, irrc::Action::Prev),
    (6, irrc::Action::Next),
    (4, irrc::Action::Rewind),
    (7, irrc::Action::Forward),
  ];
}
