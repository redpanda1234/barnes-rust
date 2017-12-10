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

impl Region {

    fn normalize_coords(&self) -> Vec<f64> {

        match self.com.clone() {

            None => vec![-1.0; DIMS],
            Some(com) => {
                let mut pos_vec = com.lock().unwrap().pos_vec.clone();
                let original = pos_vec.clone();
                for i in 0..DIMS {
                    pos_vec[i] *= 270.0 / MAX_LEN;
                    pos_vec[i] += 500.0;
                }
                // println!("original: {:?}, normalized: {:?}\n\n\n", original, pos_vec);
                pos_vec
            }

        }

    }

    fn normalize_box_coords(&self) -> Vec<f64> {

        match self.com.clone() {

            None => vec![540.0; DIMS],
            Some(com) => {
                let mut pos_vec = com.lock().unwrap().pos_vec.clone();
                let original = pos_vec.clone();
                for i in 0..DIMS {
                    pos_vec[i] *= 270.0 / MAX_LEN;
                    pos_vec[i] += 500.0;
                    pos_vec[i] -= self.half_length;
                }
                // println!("original: {:?}, normalized: {:?}\n\n\n", original, pos_vec);
                pos_vec
            }

        }

    }

    fn normalize_region_coords(&mut self) -> Vec<f64> {


        for i in 0..self.coord_vec.len() {
            self.coord_vec[i] *= 270.0 / MAX_LEN;
            self.coord_vec[i] += 400.0;
        }
        self.coord_vec.clone()


    }

}


impl Frame {

    pub fn render(&mut self, reg_option: Option<&Region>, args: &RenderArgs) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.25];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 0.05];
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
                self.render(Some(& tree), args)
            },
            Some(reg) => {
                // Draw a box rotating around the middle of the screen.
                match reg.clone().reg_vec {
                    None => {
                        self.gl.draw(args.viewport(), |c, gl| {

                        });
                        //println!("called render");
                        self.gl.draw(args.viewport(), |c, gl| {
                            let coords = reg.clone().normalize_coords();
                            if coords[0] == -1.0 {
                                return
                            } else {
                                let square = rectangle::square(0.0, 0.0, 1.0);
                                let transform = c.transform.trans(coords[0], coords[1])
                                    .rot_rad(0.0);
                                rectangle(WHITE, square, transform, gl);

                                let coords = reg.clone().normalize_box_coords();
                                let square = rectangle::square(0.0, 0.0, 2.0 * reg.half_length);
                                let transform = c.transform.trans(coords[0], coords[1]).rot_rad(0.0);
                                rectangle(GREEN, square, transform, gl);
                            }
                        });
                    },

                    Some(child_vec) => {
                        /*
                        self.gl.draw(args.viewport(), |c, gl| {
                            //draw red squares
                            let coords = reg.clone().normalize_region_coords();
                            let square = rectangle::square(0.0, 0.0, reg.half_length);
                            let transform = c.transform.trans(coords[0], coords[1])
                                            .rot_rad(0.0);
                            rectangle(GREEN, square, transform, gl);
                        });
                        */
                        for child in child_vec.iter() {
                            self.render(Some(& *child.lock().unwrap()), args);
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        self.tree.deep_update_vel();
        self.tree.deep_update_pos();
        TREE_POINTER.lock().unwrap().tree = self.tree.clone();
        self.tree.update();
        TREE_POINTER.lock().unwrap().tree = self.tree.clone();
    }
}
