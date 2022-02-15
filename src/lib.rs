#![no_std]

pub type Fix32 = fixed::FixedI32<fixed::types::extra::U16>;
pub const ZERO: Fix32 = Fix32::ZERO;
pub const ONE: Fix32 = Fix32::ONE;

#[macro_export]
macro_rules! nl {
  ($l: expr) => {
    Fix32::from_num($l)
  };
}
