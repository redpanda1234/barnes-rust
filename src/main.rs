// #[macro_use]
// extern crate itertools;

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
pub static NUMSTEPS: i16 = 10000;

lazy_static! {
    pub static ref TREE_POINTER: Mutex<Region> = Mutex::new(
        Region{
            reg_vec: None,
            coord_vec: vec![0.0; DIMS],
            half_length: 1.0,
            remove: false, // FIXME: remove?
            add_bucket: Some(vec![
                Body {
                    pos_vec: vec![-1.0, 0.0, 0.0],
                    vel_vec: vec![0.0, 0.0, 0.0],
                    mass: 1.0
                },
                Body {
                    pos_vec: vec![1.0, 0.0, 0.0],
                    vel_vec: vec![0.0, 0.0, 0.0],
                    mass: 1.0
                },
            ]),
            // add_bucket: None,
            com: None,
        }
    );
}

fn main() {
    for step in 0..NUMSTEPS {
        let printme = TREE_POINTER.lock().unwrap().update();
        println!{"printing printme {}", printme}
    }
}
