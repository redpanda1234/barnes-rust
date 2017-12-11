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

// graphics
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

// This allows us to make mutable calls to our lazy_static generated
// stuffs! Particularly useful when we're holding some global mutable
// data that all threads need to be able to access.
use std::sync::Mutex;

// define all the modules our code is in
mod data;
mod tree;
mod physics;
mod gfx;

// import all needed parts of the simulation into our current scope
pub use data::*;
pub use tree::*;
pub use physics::*;
pub use gfx::*;

use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut file = File::create("output.txt").unwrap();


    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new(
            "sim",
            [1080, 1080]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // use data::rand::SeedableRng;
    // let seed: &[_] = &[1, 2, 3, 4];
    // let seeder = SeedableRng::from_seed(seed);

    // generate the main tree. First argument gives the number of
    // masses we want to simulate, second argument passes the random
    // generation function the rng object we've just seeded. Seeding
    // is generally good while we're still in the testing phase, since
    // it gives us reproducible results.

    let num_bodies = 50;


    let root = generate::gt_all_ranges(num_bodies);

    let mut frame = Frame {
        gl: GlGraphics::new(opengl),
        tree: TREE_POINTER.lock().unwrap().tree.clone()
    };

    // println!("done generating");
    // for vec in MULTIPLIERS.lock().unwrap().clone().iter_mut() {
    //     println!("splitting multiplier: {:#?}", vec);
    // }

    let mut events = Events::new(EventSettings::new());

    let mut counter = 0;

    while let Some(e) = events.next(&mut window) {

        // make sure the tree is set up correctly before
        // trying to render or update anything
        // but actually let's not do this because what
        // really matters now is frame.tree
        //let mut tree = TREE_POINTER.lock().unwrap().tree.clone();
        //tree.update();
        //TREE_POINTER.lock().unwrap().tree = tree;

        if let Some(r) = e.render_args() {
            // println!("calling render from main");
            frame.render(None, &r);
            //println!("trying to print");
            let mut output = frame.print_masses(None);
            file.write_fmt(format_args!("{}", output));
            // println!("called render from main");
        }

        if let Some(u) = e.update_args() {
            // let frame.tree = TREE_POINTER.lock().unwrap().tree.clone();
            // TREE_POINTER.lock().unwrap().tree = frame.tree;
            // println!("calling update from main");
            frame.update(&u);
            // println!("called update from main");
        }

    }

    println!("done.");
}
