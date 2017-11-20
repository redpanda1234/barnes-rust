#[macro_use] extern crate itertools;

mod physics;
mod tree;

pub use physics::*;

// TODO: use this everywhere we check dimensions
pub const DIMS: usize = 2;

fn main() {
    // let mut static MULTIPLIERS
}
