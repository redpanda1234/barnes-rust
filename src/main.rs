#[macro_use] extern crate itertools;

mod physics;
mod tree;

pub use physics::*;
pub use tree::*;

// TODO: use this everywhere we check dimensions
pub const DIMS: usize = 2;
pub static mut TREE_POINTER: &tree::Region =
    &mut Region{
        reg_vec: None,
        coord_vec: vec![0.0; DIMS],
        half_length: 1.0,
        remove: false, // FIXME: remove?
        add_bucket: None,
        com: None,
    };

fn main() {
    // let mut static MULTIPLIERS
}
