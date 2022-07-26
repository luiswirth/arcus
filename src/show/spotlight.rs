use crate::{
  light::{
    color::{NormHsv, NormRgbw},
    controller::MemoryController,
    Lights,
  },
  return_cancel,
};
use arclib::{nl, ONE, ZERO};
use rtic::Mutex;

use super::Show;

pub struct SpotlightShow {
  light_hsv: NormHsv,
  light_pos: usize,
  light_ext: usize,
  input_state: InputState,
}
impl Default for SpotlightShow {
  fn default() -> Self {
    Self {
      light_hsv: NormRgbw::RED.into(),
      light_pos: Lights::N / 2,
      light_ext: Lights::N / 2,
      input_state: InputState::default(),
    }
  }
}
impl SpotlightShow {
  pub fn new(light_hsv: NormHsv) -> Self {
    Self {
      light_hsv,
      ..Default::default()
    }
  }
}

#[derive(Default)]
struct InputState {
  controllable: Controllable,
}

#[derive(Default)]
enum Controllable {
  #[default]
  Hue,
  Sat,
  Val,
  Pos,
  Ext,
}

impl Show for SpotlightShow {
  fn run(
    &mut self,
    cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut crate::light::controller::ColorMemoryController,
    _asm_delay: crate::util::AsmDelay,
    remote_input: &mut crate::app::shared_resources::remote_input_lock,
    config: &mut crate::app::shared_resources::config_lock,
  ) {
    // TODO: remove busy loop
    let mut time = 0;
    loop {
      // TODO: Can we avoid this lock with a channel? And is it good to do so?
      remote_input.lock(|input| match input.0.take() {
        Some(action) => {
          use crate::input::remote::Action;

          let (delta_float, delta_int) = match action {
            Action::Prev => {
              time += 1;
              (
                nl!(-0.01) + time * nl!(-0.0001),
                (nl!(-1) + time * nl!(-0.01)).ceil().to_num::<isize>(),
              )
            }
            Action::Next => {
              time += 1;
              (
                nl!(0.01) + time * nl!(0.0001),
                (nl!(1) + time * nl!(0.01)).ceil().to_num::<isize>(),
              )
            }
            _ => {
              time = 0;
              (ZERO, 0)
            }
          };

          let controllable = &mut self.input_state.controllable;
          match action {
            Action::One => *controllable = Controllable::Hue,
            Action::Two => *controllable = Controllable::Sat,
            Action::Three => *controllable = Controllable::Val,
            Action::Four => *controllable = Controllable::Pos,
            Action::Five => *controllable = Controllable::Ext,
            _ => {}
          }

          match self.input_state.controllable {
            Controllable::Hue => {
              self.light_hsv.hue = (self.light_hsv.hue + delta_float).rem_euclid(ONE);
            }
            Controllable::Sat => {
              self.light_hsv.sat = ONE.min(ZERO.max(self.light_hsv.sat + delta_float))
            }
            Controllable::Val => {
              self.light_hsv.val = ONE.min(ZERO.max(self.light_hsv.val + delta_float))
            }
            _ => {}
          };

          match self.input_state.controllable {
            Controllable::Pos => {
              self.light_pos = Lights::N.min(0.max(self.light_pos as isize + delta_int) as usize);

              let boundary_dist = self.light_pos.min(Lights::N - self.light_pos);
              self.light_ext = self.light_ext.min(boundary_dist);
            }
            Controllable::Ext => {
              self.light_ext =
                (Lights::N / 2).min(0.max(self.light_ext as isize + delta_int) as usize);
              if self.light_pos < Lights::N - self.light_pos {
                self.light_pos = self.light_pos.max(self.light_ext);
              } else {
                self.light_pos = self.light_pos.min(Lights::N - self.light_ext);
              }
            }
            _ => {}
          }
        }
        None => {}
      });

      let from = self.light_pos - self.light_ext;
      let to = self.light_pos + self.light_ext;
      for l in 0..from {
        ctrl.set(l, NormRgbw::NONE);
      }
      for l in from..to {
        ctrl.set(l, self.light_hsv.into());
      }
      for l in to..Lights::N {
        ctrl.set(l, NormRgbw::NONE);
      }
      ctrl.display(config);
      return_cancel!(cancel);
    }
  }
}
