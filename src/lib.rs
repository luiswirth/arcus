#![no_std]

pub type FixNorm = fixed::FixedU32<fixed::types::extra::U16>;
pub const ZERO: FixNorm = FixNorm::ZERO;
// TODO: check if really one
pub const ONE: FixNorm = FixNorm::MAX;

#[macro_export]
macro_rules! nl {
  ($l: expr) => {
    FixNorm::from_num($l)
  };
}
