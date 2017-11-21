#[macro_use]
extern crate itertools;

#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;

mod physics;
mod tree;

pub use physics::*;
pub use tree::*;

// TODO: use this everywhere we check dimensions
pub const DIMS: usize = 2;
pub static THETA: f64 = 0.5;
pub static DT: f64 = 0.01;
pub static NUMSTEPS: i8 = 10000;

lazy_static! {
    pub static ref TREE_POINTER: Mutex<Box<Region>> =
        Mutex::new(Box::new(
            Region{
                reg_vec: None,
                coord_vec: vec![0.0; DIMS],
                half_length: 1.0,
                remove: false, // FIXME: remove?
                add_bucket: None,
                com: None,
            }));
}

// pub static mut TREE_POINTER: &tree::Region =
//     &mut Region{
//         reg_vec: None,
//         coord_vec: vec![0.0; DIMS],
//         half_length: 1.0,
//         remove: false, // FIXME: remove?
//         add_bucket: None,
//         com: None,
//     };

fn main() {
    // for step in 0..NUMSTEPS {

    // }
}
