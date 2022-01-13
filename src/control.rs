use embedded_hal::digital::v2::InputPin;
use pico::hal::gpio::{self, Floating, Input, PullDown};

pub enum RemoteKey {
  ChannelMinus,
  Channel,
  ChannelPlus,
  Prev,
  Next,
  PlayPause,
  VolumeMinus,
  VolumePlus,
  Eq,
  Num0,
  Num100Plus,
  Num200Plus,
  Num1,
  Num2,
  Num3,
  Num4,
  Num5,
  Num6,
  Num7,
  Num8,
  Num9,
}

type IrPin = gpio::Pin<gpio::bank0::Gpio3, Input<PullDown>>;

// Arduino IR Remote Car MP3
pub struct IrRemote {
  pin: IrPin,
}

impl IrRemote {
  pub fn new(pin: IrPin) -> Self {
    Self { pin }
  }
  pub fn get_key(&self) -> Option<RemoteKey> {
    let state = self.pin.is_low().unwrap();
    match state {
      true => Some(RemoteKey::Num0),
      false => None,
    }
  }
}
