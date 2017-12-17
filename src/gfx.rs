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

pub use data::{ MIN_LEN, MAX_LEN, MAX_VEL, MAX_MASS, DIMS };

pub const screen_scale: f64 = 350.0;
pub const screen_offset: f64 = 400.0;

impl Region {


    fn normalize_coords(&self) -> Vec<f64> {

        match self.com.clone() {

            None => vec![-1.0; DIMS],
            Some(com) => {
                let mut pos_vec = com.lock().unwrap().pos_vec.clone();
                for i in 0..DIMS {
                    pos_vec[i] *= screen_scale / MAX_LEN;
                    pos_vec[i] += screen_offset;
                }
                // println!("original: {:?}, normalized: {:?}\n\n\n", original, pos_vec);
                pos_vec
            }

        }

    }

    fn liouville_normalize_coords(&self) -> Vec<f64> {

        match self.com.clone() {

            None => vec![-1.0; DIMS],

            Some(com) => {

                let mut pos_mag = 0.0;
                let mut mom_mag = 0.0;

                // Destructure body
                let Body {
                    pos_vec: pos_vec,
                    vel_vec: vel_vec,
                    mass: mass
                } = com.lock().unwrap().clone();

                let mut pos_vec = com.lock().unwrap().pos_vec.clone();
                let mut vel_vec = com.lock().unwrap().vel_vec.clone();
                let mass = com.lock().unwrap().mass.clone();

                for i in 0..DIMS {
                    pos_vec[i] *= screen_scale / MAX_LEN;
                    pos_vec[i] += screen_offset;

                    // 4.0 below chosen arbitrarily to just make things not fly as far away
                    vel_vec[i] *= mass * screen_scale / (4.0 * MAX_VEL * MAX_MASS);
                    vel_vec[i] += screen_offset;

                    pos_mag += pos_vec[i].powi(2);
                    mom_mag += vel_vec[i].powi(2);
                }
                vec![pos_mag.sqrt(), mom_mag.sqrt()]
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

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.05];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 0.05];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 0.05];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];

        //main should pass render() a None option
        //if that happens, call render on the tree
        match reg_option {

            None => {
                let tree = TREE_POINTER.lock().unwrap().tree.clone();

                self.gl.draw(args.viewport(), |c, gl| {
                    // Clear the screen.
                    //clear(BLACK, gl);
                });

                // self.gl.draw(args.viewport(), |c, gl| {
                //     let coords = [screen_offset, screen_offset];
                //     let square = rectangle::square(0.0, 0.0, 4.0);
                //     let transform = c.transform.trans(coords[0], coords[1]).rot_rad(0.0);
                //     rectangle(RED, square, transform, gl);
                // });

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

                                let transform =
                                    c.transform
                                    .trans(coords[0], coords[1])
                                    .rot_rad(0.0);

                                match reg.com.clone() {
                                    None => (),
                                    Some (com) => {
                                        if com.lock().unwrap().clone().mass < 9000.0 {
                                            let square = rectangle::square(0.0, 0.0, 2.0);
                                            rectangle(WHITE, square, transform, gl);
                                        } else {
                                            let square = rectangle::square(0.0, 0.0, 4.0);
                                            rectangle(YELLOW, square, transform, gl);
                                        }
                                    }
                                }

                                // Optional drawing of green squares representing regions with children

                                // let coords = reg.normalize_region_coords();

                                // let square = rectangle::square(0.0, 0.0, 2.0*reg.half_length * (screen_scale / MAX_LEN));
                                // let transform = c.transform.trans(coords[0], coords[1]).rot_rad(0.0);
                                // rectangle(GREEN, square, transform, gl);
                            }
                        });
                    },

                    Some(child_vec) => {
                        //
                        // Optional drawing of blue squares representing current regions

                        // self.gl.draw(args.viewport(), |c, gl| {
                        //     //draw red squares
                        //     let coords = reg.clone().normalize_region_coords();
                        //     let square = rectangle::square(0.0, 0.0, 2.0*reg.half_length * (screen_scale) / MAX_LEN);
                        //     let transform = c.transform.trans(coords[0], coords[1]).rot_rad(0.0);
                        //     rectangle(BLUE, square, transform, gl);
                        // });
                        // self.gl.draw(args.viewport(), |c, gl| {
                        //     match reg.com.clone() {
                        //         None => (),
                        //         Some(com) => {
                        //             let mut mass = com.lock().unwrap().mass;
                        //             if mass > 0.0 {

                        //                 let coords = reg.clone().normalize_coords();

                        //                 if coords[0] == -1.0 {
                        //                 } else {
                        //                     let square = rectangle::square(0.0, 0.0, 1.0);

                        //                     // let transform =
                        //                     //     c.transform
                        //                     //     .trans(coords[0], coords[1])
                        //                     //     .rot_rad(0.0);

                        //                     // rectangle(RED, square, transform, gl);

                        //                 }
                        //             };
                        //         }
                        //     }
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

    pub fn phase_render(&mut self, reg_option: Option<&Region>, args: &RenderArgs) {
        // println!("phase_render");
        use graphics::*;

        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 0.5];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

        match reg_option {

            None => {
                let tree = TREE_POINTER.lock().unwrap().tree.clone();

                self.gl.draw(args.viewport(), |c, gl| {
                    // Clear the screen.
                    clear(BLACK, gl);
                });

                //todo: replace this with drawing a red square at the master tree's com
                //println!("none option passed to render; mass: {:#?}", tree.com.mass);
                self.phase_render(Some(&tree), args)
            },

            Some(reg) => {

                match reg.clone().reg_vec {

                    None => {
                        self.gl.draw(args.viewport(), |c, gl| {
                            let coords = reg.clone().liouville_normalize_coords();
                            // println!("{:?}", coords);
                            if coords[0] == -1.0 {

                                return

                            } else {

                                let transform =
                                    c.transform
                                    .trans(coords[0], coords[1])
                                    .rot_rad(0.0);

                                match reg.com.clone() {
                                    None => (),
                                    Some (com) => {
                                        let square = rectangle::square(0.0, 0.0, 1.0);
                                        rectangle(WHITE, square, transform, gl);
                                    }
                                }
                            }
                        });
                    },

                    Some(child_vec) => {
                        for child in child_vec.iter() {
                            self.phase_render(
                                Some(& *child.lock().unwrap()),
                                args
                            );
                        }
                    }
                }
            }
        }
    }


    pub fn print_masses(&mut self, reg_option: Option<&Region>) -> String {
        //recurse into the subregions
        // let reg_option = self.tree.reg_vec.clone();
        let mut output = String::new();
        match reg_option {

            None => {
                let tree = TREE_POINTER.try_lock().unwrap().tree.clone();
                // println!("let tree");
                output = self.print_masses(Some(&tree));
                output.push_str(&format!("\n"));
                output
            },


            Some(reg) => {
                // println!("entered Some arm");
                match reg.reg_vec.clone() {

                    None => {
                        match reg.com.clone() {
                            None => output,
                            Some(our_reg) => {
                                let mass = our_reg.try_lock().unwrap().clone();

                                if mass.mass == 100000.01 {
                                    output.push_str(&format!("\t{:#?}", mass.mass));
                                    output.push_str(&format!("\t{:#?}", mass.pos_vec[0]));
                                    output.push_str(&format!("\t{:#?}", mass.pos_vec[1]));
                                    output.push_str(&format!("\t{:#?}", mass.vel_vec[0]));
                                    output.push_str(&format!("\t{:#?}", mass.vel_vec[1]));
                                }
                                // println!("{}", output);
                                output
                            }
                        }
                    },

                    Some(child_vec) => {

                        for child in child_vec.iter() {
                            //let mut clone = output.clone();
                            output.push_str(
                                &self.print_masses(
                                    Some(& *child.lock().unwrap()),
                                )
                            );//
                        }
                        output
                    }
                }
            }
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        // self.tree = TREE_POINTER.lock().unwrap().tree.clone();
        // self.tree.update();
        // TREE_POINTER.lock().unwrap().tree = self.tree.clone();

        // let mut output = String::new();
        // output =  self.print_masses(None, output);
        // println!("{}", output);
        self.tree.deep_update_vel();
        TREE_POINTER.lock().unwrap().tree = self.tree.clone();
        self.tree.deep_update_pos();
        self.tree.add_queue = TREE_POINTER.lock().unwrap().tree.add_queue.clone();
        TREE_POINTER.lock().unwrap().tree = self.tree.clone();
        self.tree.update();
    }
}
