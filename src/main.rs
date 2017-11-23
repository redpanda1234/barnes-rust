// #[macro_use]
// extern crate itertools;

// #[macro_use] tells Rust to also import defined macros from the
// crate we're looking at.

// lazy_static allows us to generate static global variables at
// runtime. This is incredibly useful, as it allows us to generalize
// our simulation algorithm to higher dimensions, because we can
// generate our MULTIPLIERS vector at runtime. 
#[macro_use]
extern crate lazy_static;

// This allows us to make mutable calls to our lazy_static generated
// stuffs! Particularly useful when we're holding some global mutable
// data that all threads need to be able to access. 
use std::sync::Mutex;

// define all the modules our code is in 
mod data;
mod tree;
mod physics;

// import all needed parts of the simulation into our current scope 
pub use data::*;
pub use tree::*;
pub use physics::*;

fn main() {
    for step in 0..NUMSTEPS {
        TREE_POINTER.lock().unwrap().update();
        let printme = &TREE_POINTER.lock().unwrap().clone().reg_vec;
        println!{"printing printme {:?}", printme};
    }
}
