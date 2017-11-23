// #[macro_use]
// extern crate itertools;

#[warn(unused_variables)]

#[macro_use]
extern crate lazy_static;

use std::sync::Mutex;
use gen_mult::populate_mult;
mod physics;
mod tree;

pub use physics::*;
pub use tree::*;

// TODO: use this everywhere we check dimensions
pub const DIMS: usize = 3;
pub static THETA: f64 = 0.5;
pub static DT: f64 = 0.01;
pub static NUMSTEPS: i16 = 10;
pub static mut NUM_THREADS: i64 = 20;

pub mod gen_mult {
    
    pub fn populate_mult(n: usize, mult: f64) -> Vec<Vec<f64>> {
        if n <= 0 {
            return vec![vec![mult]];
        }

        let mut v1: Vec<Vec<f64>> = populate_mult(n - 1, -1.0);
        v1.extend(populate_mult(n - 1, 1.0));

        if mult != 0.0 {
            for i in 0..v1.len() {
                v1[i].push(mult);
            }
        }

        v1
    }
}



lazy_static! {    
    pub static ref TREE_POINTER: Mutex<Region> = Mutex::new(
        Region{
            reg_vec: None,
            coord_vec: vec![0.0; DIMS],
            half_length: 1.0,
            remove: false, // FIXME: remove?
            add_bucket: Some(vec![
                Body {
                    pos_vec: vec![-0.5, 0.0, 0.0],
                    vel_vec: vec![0.0, 0.0, 0.0],
                    mass: 1.0
                },
                Body {
                    pos_vec: vec![0.5, 0.0, 0.0],
                    vel_vec: vec![0.0, 0.0, 0.0],
                    mass: 1.0
                },
            ]),
            // add_bucket: None,
            com: None,
        }
    );

    pub static ref MULTIPLIERS: Mutex<Vec<Vec<f64>>> = Mutex::new(
        populate_mult(DIMS, 0.0)
    );
}

fn main() {
    for step in 0..NUMSTEPS {
        TREE_POINTER.lock().unwrap().update();
        let printme = &TREE_POINTER.lock().unwrap().clone().reg_vec;
        println!{"printing printme {:?}", printme};
    }
}
