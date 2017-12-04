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

static NUMSTEPS: usize = 1000;

fn main() {

    use data::rand::SeedableRng;
    let seed: &[_] = &[1, 2, 3, 4];
    let seeder = SeedableRng::from_seed(seed);

    // generate the main tree. First argument gives the number of
    // masses we want to simulate, second argument passes the random
    // generation function the rng object we've just seeded. Seeding
    // is generally good while we're still in the testing phase, since
    // it gives us reproducible results.

    generate::gt_all_ranges(5, seeder);

    println!("done generating");

    // unsafe {
        for _ in 0..NUMSTEPS {
            // let printme = TREE_POINTER.lock().unwrap().tree.clone();
            // println!{"printing printme \n{:#?}\n\n\n\n\n\n", printme};
            let mut tree = TREE_POINTER.lock().unwrap().tree.clone();
            tree.update();
            tree.deep_update_vel();
            tree.deep_update_pos();
            TREE_POINTER.lock().unwrap().tree = tree;
        }
    // }

    println!("done.");
}
