use alloc::vec::Vec;
use rand::{prelude::Distribution, Rng, SeedableRng};

use crate::{
  app::monotonics,
  light::{
    color::{NormHsv, NormRgbw},
    controller::{ColorMemoryController, MemoryController, MemoryControllerExt},
    Lights,
  },
  return_cancel,
  util::AsmDelay,
};

use super::Show;

struct Snake {
  pub pos: usize,
  pub tail: Vec<NormRgbw>,
}

struct Fruit {
  pub pos: usize,
  pub color: NormRgbw,
}

#[derive(Default)]
pub struct SnakeShow;
impl Show for SnakeShow {
  fn run(
    &mut self,
    cancel: &mut crate::app::shared_resources::show_cancellation_token_lock,
    ctrl: &mut ColorMemoryController,
    _asm_delay: AsmDelay,
    _remote_input: &mut crate::app::shared_resources::remote_input_lock,
    config: &mut crate::app::shared_resources::config_lock,
  ) {
    let mut rng = rand::rngs::SmallRng::seed_from_u64(monotonics::now().ticks());
    let pos_distr = rand::distributions::Uniform::new(0, Lights::N);
    let mut snake = Snake {
      pos: pos_distr.sample(&mut rng),
      tail: Vec::new(),
    };
    let mut fruit = Fruit {
      pos: pos_distr.sample(&mut rng),
      color: rng.gen::<NormHsv>().into(),
    };

    loop {
      ctrl.set_all(NormRgbw::NONE);
      if snake.pos == fruit.pos {
        snake.tail.insert(0, fruit.color);
        // TODO: don't spawn fruits inside snake
        fruit.pos = pos_distr.sample(&mut rng);
        fruit.color = rng.gen::<NormHsv>().into();
      }
      for (i, &segment) in snake.tail.iter().enumerate() {
        let pos = (snake.pos + i) % Lights::N;
        ctrl.set(pos, segment);
      }
      ctrl.set(fruit.pos, fruit.color);
      ctrl.display(config);
      snake.pos = (snake.pos + Lights::N - 1) % Lights::N;
      return_cancel!(cancel);
    }
  }
}
