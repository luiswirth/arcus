use crate::{
  app::remote_task::{self, SharedResources},
  light::color::Color,
  show::{self, Show},
};

use alloc::boxed::Box;
use infrared::{self as ir, remotecontrol as irrc};
use rp_pico::hal::gpio;
use rtic::Mutex;

type IrProto = infrared::protocol::Nec;
type IrCommand = <IrProto as infrared::Protocol>::Cmd;
pub type IrReceiverPin = gpio::Pin<gpio::bank0::Gpio3, gpio::Input<gpio::Floating>>;
pub type IrReceiver = infrared::Receiver<
  IrProto,
  ir::receiver::Event,
  ir::receiver::PinInput<IrReceiverPin>,
  irrc::Button<NadRc512>,
>;

pub struct RemoteTask {
  ir_receiver: IrReceiver,
  last_event_instant: u32,
}
impl RemoteTask {
  pub fn init(ir_pin: IrReceiverPin) -> Self {
    ir_pin.set_interrupt_enabled(gpio::Interrupt::EdgeHigh, true);
    ir_pin.set_interrupt_enabled(gpio::Interrupt::EdgeLow, true);

    let ir_receiver: IrReceiver = ir::Receiver::builder()
      .pin(ir_pin)
      .event_driven()
      .protocol::<IrProto>()
      .remotecontrol(NadRc512)
      .build();

    Self {
      ir_receiver,
      last_event_instant: 0,
    }
  }
}

pub fn remote_task(ctx: remote_task::Context) {
  let RemoteTask {
    ir_receiver,
    last_event_instant,
  } = ctx.local.remote_task;
  let SharedResources {
    mut timer,
    mut show,
    uart: _uart,
  } = ctx.shared;

  let pin = ir_receiver.pin();
  pin.clear_interrupt(gpio::Interrupt::EdgeHigh);
  pin.clear_interrupt(gpio::Interrupt::EdgeLow);

  let now = timer.lock(|t| t.get_counter_low());
  let dt = now.wrapping_sub(*last_event_instant);
  *last_event_instant = now;

  match ir_receiver.event(dt) {
    Ok(Some(cmd)) => match cmd.action() {
      Some(action) => match next_show(action) {
        Some(next_show) => show.lock(|show| *show = Some(next_show)),
        None => {}
      },
      None => {}
    },
    // TODO: handle
    Err(_e) => {}
    Ok(None) => {}
  };
}

fn next_show(action: irrc::Action) -> Option<Box<dyn Show + Send>> {
  macro_rules! uni {
    ($c:path) => {
      Some(Box::new(show::UniformShow::new($c)))
    };
  }

  match action {
    irrc::Action::Stop => uni!(Color::NONE),
    irrc::Action::Play_Pause => None,
    irrc::Action::Random => None,
    irrc::Action::Time => None,
    irrc::Action::Repeat => None,
    irrc::Action::One => uni!(Color::RED),
    irrc::Action::Two => uni!(Color::GREEN),
    irrc::Action::Three => uni!(Color::BLUE),
    irrc::Action::Four => uni!(Color::YELLOW),
    irrc::Action::Five => uni!(Color::MAGENTA),
    irrc::Action::Six => uni!(Color::CYAN),
    irrc::Action::Seven => uni!(Color::WHITE),
    irrc::Action::Eight => uni!(Color::RGB),
    irrc::Action::Nine => uni!(Color::RGBW),
    //irrc::Action::? => None,
    irrc::Action::Zero => None,
    irrc::Action::Prog => None,
    irrc::Action::Prev => None,
    irrc::Action::Next => None,
    irrc::Action::Rewind => None,
    irrc::Action::Forward => None,
    _ => None,
  }
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
    //(77, irrc::Action::?),
    (76, irrc::Action::Zero),
    (11, irrc::Action::Prog),
    (5, irrc::Action::Prev),
    (6, irrc::Action::Next),
    (4, irrc::Action::Rewind),
    (7, irrc::Action::Forward),
  ];
}
