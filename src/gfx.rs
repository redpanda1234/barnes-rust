extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

use super::tree::*;
use super::data::TREE_POINTER;

use std::sync::{Arc, Mutex};

pub struct Frame {
    pub gl: GlGraphics, // OpenGL backend for drawing
    pub tree: Region // the tree we're gonna be drawing
}

pub use data::{ MAX_LEN, DIMS };

pub const screen_scale: f64 = 270.0;
pub const screen_offset: f64 = 300.0;

impl Region {


    fn normalize_coords(&self) -> Vec<f64> {

        match self.com.clone() {

            None => vec![-1.0; DIMS],
            Some(com) => {
                let mut pos_vec = com.lock().unwrap().pos_vec.clone();
                let original = pos_vec.clone();
                for i in 0..DIMS {
                    pos_vec[i] *= screen_scale / MAX_LEN;
                    pos_vec[i] += screen_offset;
                }
                // println!("original: {:?}, normalized: {:?}\n\n\n", original, pos_vec);
                pos_vec
            }

        }

    }

    fn normalize_region_coords(&self) -> Vec<f64> {

        let mut coord_vec = self.coord_vec.clone();

        for i in 0..DIMS {
            coord_vec[i] *= screen_scale / MAX_LEN;
            coord_vec[i] += screen_offset;
            coord_vec[i] -= self.half_length * (screen_scale / MAX_LEN);
        }

        coord_vec

    }

}


impl Frame {

    pub fn render(&mut self, reg_option: Option<&Region>, args: &RenderArgs) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.25];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 0.05];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 0.1];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        //main should pass render() a None option
        //if that happens, call render on the tree
        match reg_option {

            None => {
                let tree = TREE_POINTER.lock().unwrap().tree.clone();

                self.gl.draw(args.viewport(), |c, gl| {
                    // Clear the screen.
                    clear(BLACK, gl);
                });

                self.gl.draw(args.viewport(), |c, gl| {
                    let coords = [500.0, 500.0];
                    let square = rectangle::square(0.0, 0.0, 2.0);
                    let transform = c.transform.trans(coords[0], coords[1]).rot_rad(0.0);
                    rectangle(RED, square, transform, gl);
                });

                //todo: replace this with drawing a red square at the master tree's com
                //println!("none option passed to render; mass: {:#?}", tree.com.mass);
                self.render(Some(& tree), args)

            },
            Some(reg) => {

                match reg.clone().reg_vec {

                    None => {
                        self.gl.draw(args.viewport(), |c, gl| {
                            let coords = reg.clone().normalize_coords();

                            if coords[0] == -1.0 {

                                return

                            } else {
                                let square = rectangle::square(0.0, 0.0, 1.0);

                                let transform =
                                    c.transform
                                    .trans(coords[0], coords[1])
                                    .rot_rad(0.0);

                                rectangle(WHITE, square, transform, gl);

                                let coords = reg.normalize_region_coords();

                                let square = rectangle::square(0.0, 0.0, 2.0*reg.half_length * (screen_scale / MAX_LEN));
                                let transform = c.transform.trans(coords[0], coords[1]).rot_rad(0.0);
                                rectangle(GREEN, square, transform, gl);
                            }
                        });
                    },

                    Some(child_vec) => {
                        //
                        // self.gl.draw(args.viewport(), |c, gl| {
                        //     //draw red squares
                        //     let coords = reg.clone().normalize_region_coords();
                        //     let square = rectangle::square(0.0, 0.0, 2.0*reg.half_length * (screen_scale) / MAX_LEN);
                        //     let transform = c.transform.trans(coords[0], coords[1]).rot_rad(0.0);
                        //     rectangle(BLUE, square, transform, gl);
                        // });
                        for child in child_vec.iter() {
                            self.render(
                                Some(& *child.lock().unwrap()),
                                args
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        self.tree.deep_update_vel();
        self.tree.deep_update_pos();
        println!("in gfx update, TREE add queue: {:#?}", TREE_POINTER.lock().unwrap().tree.add_queue.clone());
        self.tree.add_queue = TREE_POINTER.lock().unwrap().tree.add_queue.clone();
        println!("in gfx update, our add queue: {:#?}", self.tree.add_queue.clone().unwrap());
        TREE_POINTER.lock().unwrap().tree = self.tree.clone();
        self.tree.update();
        TREE_POINTER.lock().unwrap().tree = self.tree.clone();
    }
}
