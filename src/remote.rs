use crate::app::{
  self, input_task, monotonics,
  remote_task::{self, SharedResources},
};
use infrared::{self as ir, remotecontrol as irrc};
use rp_pico::hal::gpio;
use rtic::Mutex;

#[derive(Debug, Default)]
pub struct RemoteInput(pub Option<irrc::Action>);

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
  let RemoteTask { ir_receiver } = ctx.local.remote_task;
  let SharedResources { mut remote_input } = ctx.shared;

  let now = monotonics::now();
  match ir_receiver.event_instant(now) {
    Ok(Some(cmd)) => match cmd.action() {
      Some(action) => {
        remote_input.lock(|input| {
          input.0 = Some(action);
        });
        input_task::spawn().unwrap();
      }
      None => {}
    },
    Err(_) => unreachable!("This should be infalliable."),
    Ok(None) => {}
  };

  let pin = ir_receiver.pin_mut();
  pin.clear_interrupt(gpio::Interrupt::EdgeHigh);
  pin.clear_interrupt(gpio::Interrupt::EdgeLow);
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
    // TODO: Find better fitting Action.
    // NOTE: This is the "+10" Button.
    (77, irrc::Action::Teletext),
    (76, irrc::Action::Zero),
    (11, irrc::Action::Prog),
    (5, irrc::Action::Prev),
    (6, irrc::Action::Next),
    (4, irrc::Action::Rewind),
    (7, irrc::Action::Forward),
  ];
}
