use arclib::{ONE, ZERO};
use rp_pico::hal::timer::CountDown;

use crate::light::{
  color::Color,
  controller::{MemoryController, MemoryControllerExt, U32Memory, U32MemoryController},
  show::State,
  Lights,
};

use super::Show;

pub struct DemoShow {
  state: DemoState,
  memory: U32Memory,
}
impl Default for DemoShow {
  fn default() -> Self {
    Self {
      state: DemoState::Init,
      memory: U32Memory::new(),
    }
  }
}

pub enum DemoState {
  Init,
  SingleChannel { c: usize, l: usize },
  Rgb { l: usize },
  Rgbw { l: usize },
  //HSV(usize),
}

impl Show for DemoShow {
  fn update(&mut self, lights: &mut Lights, count_down: CountDown) -> State {
    const N: usize = Lights::N;
    let mut ctrl = U32MemoryController::new(lights, &mut self.memory, count_down);

    match self.state {
      DemoState::Init => {
        ctrl.set_all(Color::NONE);
        self.state = DemoState::SingleChannel { c: 0, l: 0 };
        State::Running
      }
      DemoState::SingleChannel { mut c, mut l } => {
        let mut color = Color::NONE;
        color[c] = ONE;
        ctrl.set(l, color);
        ctrl.display();
        //utils.delay_ms(40);
        l += 1;
        if l == N {
          l = 0;
          c += 1;
        }
        if c == 4 {
          self.state = DemoState::Rgb { l };
        } else {
          self.state = DemoState::SingleChannel { c, l };
        }
        State::Running
      }
      DemoState::Rgb { mut l } => {
        let color = Color::new(ONE, ONE, ONE, ZERO);
        ctrl.set(l, color);
        ctrl.display();
        //utils.delay_ms(40);
        l += 1;
        if l == N {
          l = 0;
          self.state = DemoState::Rgbw { l };
        } else {
          self.state = DemoState::Rgb { l };
        }
        State::Running
      }
      DemoState::Rgbw { mut l } => {
        let color = Color::new(ONE, ONE, ONE, ONE);
        ctrl.set(l, color);
        ctrl.display();
        //utils.delay_ms(40);
        l += 1;
        if l == N {
          State::Finished
        } else {
          self.state = DemoState::Rgbw { l };
          State::Running
        }
      }
    }

    //for pass in 0..2 {
    //  for l in 0..N {
    //    let hue = l as f32 / N as f32;
    //    let color = Color::from_hsv(hue, 1.0, 1.0);
    //    if pass == 0 && l != 0 {
    //      ctrl.set(l - 1, Color::NONE);
    //    }
    //    ctrl.set(l, color);
    //    ctrl.display();
    //    utils.delay_ms(40);
    //  }
    //}

    //let mut hue = 0.0;
    //loop {
    //  hue += 0.01;
    //  if hue > 1.0 {
    //    break;
    //  }
    //  for l in 0..N {
    //    let color = Color::from_hsv(hue, 1.0, 1.0);
    //    ctrl.set(l, color);
    //  }
    //  ctrl.display();
    //  utils.delay_ms(40);
    //}
  }
}
