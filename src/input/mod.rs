pub mod remote;

use alloc::boxed::Box;
use arclib::nl;
use infrared::remotecontrol::Action;
use rtic::Mutex;

use crate::{
  app::input_task::{self, SharedResources},
  light::color::NormColor,
  show::{self, Show},
};

#[derive(Default)]
pub struct InputTask {
  state: InputState,
}
pub enum InputState {
  Waiting,
  ShowSelection,
  BrightnessSelection,
}
impl Default for InputState {
  fn default() -> Self {
    Self::Waiting
  }
}

pub fn input_task(ctx: input_task::Context) {
  let InputTask { state } = ctx.local.input_task;
  let SharedResources {
    mut remote_input,
    mut config,
    mut show_cancellation_token,
  } = ctx.shared;

  let remote_action = remote_input
    .lock(|input| input.0.take())
    .expect("input_task should only be called when there is input");

  match &state {
    InputState::Waiting => match remote_action {
      Action::Stop => {
        show_cancellation_token.lock(|cancel| cancel.request());
      }
      Action::Play_Pause => {
        *state = InputState::ShowSelection;
      }
      Action::Time => {
        *state = InputState::BrightnessSelection;
      }
      Action::Rewind => {
        config.lock(|config| config.brightness = (config.brightness - nl!(0.05)).max(nl!(0.0)));
      }
      Action::Forward => {
        config.lock(|config| config.brightness = (config.brightness + nl!(0.05)).min(nl!(1.0)));
      }
      _ => {
        remote_input.lock(|input| input.0 = Some(remote_action));
      }
    },
    InputState::ShowSelection => {
      if let Some(next_show) = next_show(remote_action) {
        config.lock(|config| config.show = Some(next_show));
        show_cancellation_token.lock(|cancel| cancel.request());
        *state = InputState::Waiting;
      }
    }
    InputState::BrightnessSelection => {
      if let Some(brightness) = number_from_action(remote_action) {
        let brightness = nl!(brightness) / nl!(9);
        config.lock(|config| config.brightness = brightness);
        *state = InputState::Waiting;
      }
    }
  }
}

#[rustfmt::skip]
fn number_from_action(action: Action) -> Option<usize> {
  match action {
    Action::Zero  => Some(0),
    Action::One   => Some(1),
    Action::Two   => Some(2),
    Action::Three => Some(3),
    Action::Four  => Some(4),
    Action::Five  => Some(5),
    Action::Six   => Some(6),
    Action::Seven => Some(7),
    Action::Eight => Some(8),
    Action::Nine  => Some(9),
    _ => None,
  }
}

fn color_from_action(action: Action) -> Option<NormColor> {
  number_from_action(action).map(|i| {
    if i == 0 {
      NormColor::NONE
    } else {
      NormColor::STANDARD_PALETTE[i - 1]
    }
  })
}

#[rustfmt::skip]
fn next_show(action: Action) -> Option<Box<dyn Show + Send>> {
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
      Action::Stop       => show!(show::NullShow::default()),
      Action::Prog       => show!(show::DemoShow::default()),
      Action::Time       => show!(show::SeparatedClockShow::default()),
      Action::Random     => show!(show::RandomShow::default()),
      Action::Teletext   => show!(show::SnakeShow::default()),
      Action::Repeat     => show!(show::ByteShow::new(BYTES)),
      Action::Prev       => None,
      Action::Next       => None,
      Action::Rewind     => None,
      Action::Forward    => None,
      _ => None,
    }
  }
}
