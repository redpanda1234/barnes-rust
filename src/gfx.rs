extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

use super::tree*;

pub struct Frame {
    gl: GlGraphics, // OpenGL backend for drawing
    tree: Tree // the tree we're gonna be drawing
}

pub use data::{ MAX_LEN, DIMS };

impl Body {

    fn normalize_coords(&self) -> Vec<f64> {

        let mut clone = self.com.clone();

        match clone {

            None => vec![0.0; DIMS],
            Some(mut com) => {
                for i in 0..com.pos_vec.len() {
                    com.pos_vec[i] *= 1080.0 / MAX_LEN;
                }
                com.pos_vec
            }

        }

    }

}

impl Frame {

    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const BLACK: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const WHITE: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

        // let square = rectangle::square(0.0, 0.0, 1080.0);
        // let rotation = self.rotation;
        // let (x, y) = ((args.width / 2) as f64,
        //               (args.height / 2) as f64);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            // let transform = c.transform.trans(x, y)
            //                            .rot_rad(rotation)
            //                            .trans(-25.0, -25.0);

            // Draw a box rotating around the middle of the screen.
            let mut tree_clone = self.tree.clone();
            match tree_clone.reg_vec() {

                None => {
                    let coords = self.com.normalize_coords();
                    let square = rectangle::square(coords[0], coords[1], 1.0 );
                    rectangle(WHITE, square, gl);
                },

                Some(child_vec) => {
                    for child in child_vec.iter_mut() {
                        child.render();
                    }
                }
            }
            // rectangle(RED, square, transform, gl);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.tree.update();
        self.tree.deep_update_vel();
        self.tree.deep_update_pos();
    }
}
