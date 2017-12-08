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

pub struct Frame {
    pub gl: GlGraphics, // OpenGL backend for drawing
    pub tree: Region // the tree we're gonna be drawing
}

#[derive(Debug, Clone)]
pub struct Pixel {
    //gl: GlGraphics,
    normalized_coords: Vec<f64>
}

// pub mod Omg {
//     use super::opengl_graphics::{ GlGraphics, OpenGL };
//     use super::Pixel;
pub fn new_pixel(normalized_coords: Vec<f64>) -> Pixel {

    Pixel {
        //gl: GlGraphics::new(OpenGL::V3_2),
        normalized_coords: normalized_coords
    }

}
// }

pub use data::{ MAX_LEN, DIMS };

impl Region {

    fn normalize_coords(self) -> Vec<f64> {

        match self.com.clone() {

            None => vec![0.0; DIMS],
            Some(mut com) => {
                for i in 0..com.pos_vec.len() {
                    com.pos_vec[i] *= 270.0 / MAX_LEN;
                    com.pos_vec[i] += 270.0;
                }
                com.pos_vec
            }

        }

    }

}

impl Pixel {

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const WHITE: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 0.0];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 1080.0);
    }

}

impl Frame {

    pub fn render(&mut self, reg_option: Option<&Region>, args: &RenderArgs) {
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.5];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        //let square = rectangle::square(0.0, 0.0, 1080.0);

        //main should pass render() a None option
        //if that happens, call render on the tree
        match reg_option {
            None => {
                let tree = self.tree.clone();
                self.gl.draw(args.viewport(), |c, gl| {
                    // Clear the screen.
                    clear(BLACK, gl);
                });
                self.render(Some(& tree), args)
            },
            Some (reg) => {
                // Draw a box rotating around the middle of the screen.
                match reg.clone().reg_vec {
                    None => {
                        //println!("called render");
                        self.gl.draw(args.viewport(), |c, gl| {
                            //draw red squares
                            let coords = reg.clone().normalize_coords();
                            let square = rectangle::square(0.0, 0.0, 1.0);
                            let transform = c.transform.trans(coords[0], coords[1])
                                            .rot_rad(0.0);
                            rectangle(WHITE, square, transform, gl);
                        });
                    },

                    Some(child_vec) => {
                        for child in child_vec.iter() {
                            self.render(Some(& *child), args);
                        }
                    }
                }
            }
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        self.tree.update();
        TREE_POINTER.lock().unwrap().tree = self.tree.clone();
        // println!("{:#?}", self.tree);
        self.tree.deep_update_vel();
        TREE_POINTER.lock().unwrap().tree = self.tree.clone();
        self.tree.deep_update_pos();
    }
}
