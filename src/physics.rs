// The physics module is really just a collection of methods on
// structs defined in tree that we've clustered here because they all
// have to do with the actual physics part of the simulation. So we
// have to move up one level (super), and import tree::*
pub use super::tree::*;

// Fetch global statics from the main function
pub use super::data::{DIMS, TREE_POINTER, DT, THETA};

impl Body {

    // We need r^2 in Newton's law of gravity (TODO: apply small GR
    // perturbation), and it's faster to have separately defined
    // methods for this and finding the magnitude of the displacement
    // vector between the two bodies, because of the way we're
    // constructing net accelerations.

    // the squared_dist_to method takes a reference to a Body struct,
    // and iterates through pairs of coordinates in the calling Body's
    // position and the passed mass's position to return r^2.

    pub fn squared_dist_to(&self, mass: &Body) -> f64 {
        let mut r_squared: f64 = 0.0;
        for (qi, pi) in self.pos_vec.iter().zip(&mass.pos_vec) {
            r_squared += (qi - pi).powi(2);
        }
        r_squared
    }

    // vec_rel gets the displacement vector between the calling mass
    // and some other passed Body.

    pub fn vec_rel(&self, mass: &Body, dims: usize) -> Vec<f64> {
        let mut vec: Vec<f64> = vec![0.0; dims];
        for i in 0..dims {
            vec[i] = mass.pos_vec[i] - self.pos_vec[i];
        }
        vec
    }

    // sq_magnitude should really probably just be its own function,
    // there's really no need to define it as a method here. TODO: fix
    // this, by moving it to a separate module? It takes as input some
    // vector, and finds the square of its magnitude. This is helpful
    // to define separately from squared_dist_to, even though they're
    // functionally equivalent (but this is slower), because we don't
    // always need to find the displacement vector.

    pub fn sq_magnitude(&self, vec: &Vec<f64>, dims: usize) -> f64 {
        let mut r_squared: f64 = 0.0;
        for i in 0..dims {
            r_squared += vec[i].powi(2)
        }
        r_squared
    }

    // is_far calculates a distance metric between the calling mass
    // (which really should only ever be the com of a leaf node in the
    // tree) and a passed region.

    pub fn is_far(&self, node: &mut Region) -> bool {
        // this makes me think we should store full-length instead of
        // half-length FIXME
        match node.com.clone() {
            // FIXME: make sure this doesn't allow infinite loops;
            // i.e. that node.com will only be none if there's stuff
            // in the region_vec or add_queue.
            None => {node.update_com(); self.is_far(node)},
            Some(_com) => {
                ( 2.0 * node.half_length /
                  self.squared_dist_to(&node.com.clone().unwrap()) )
                    <= THETA
            }
        }
    }

    pub fn get_classical_accel(&self, mass: &Body, dims: usize) -> Vec<f64> {
        let mut rel = self.vec_rel(mass, dims);
        let sq_mag = self.sq_magnitude(&rel, dims);
        println!("{}, {:#?}", sq_mag, rel);
        let acc = mass.mass * (6.674 / (1_000_000_000_00.0)) / sq_mag;
        let r = sq_mag.sqrt();

        for i in 0..dims {
            // TODO: make this work for generic number of dimensions
            // vec.push(self.pos_vec[i] * acc / r) // pos_vec[i]/r is trig
            rel[i] *= acc/r;
        }
        println!("{:#?}", rel);
        rel
    }

    pub fn update_accel(&self, mut acc: Vec<f64>, mass: &Body, dims: usize ) -> Vec<f64> {
        for (mut acci, ai) in acc.iter_mut().zip(
            self.get_classical_accel(mass, dims)) {
            *acci += ai;
        }
        acc
    }

    pub fn get_total_acc(&mut self, node: &mut Region) -> Vec<f64> {
        let mut acc = vec![0.0; DIMS];
        match node.reg_vec.clone() {
            None => self.update_accel(acc, &node.com.clone().unwrap(), DIMS),
            Some(ref reg_vec) => {
                if self.is_far(node) {
                    self.update_accel(acc, &node.com.clone().unwrap(), DIMS)
                } else {
                    for child in reg_vec.iter() {
                        acc = self.update_accel(
                            acc, &child.com.clone().unwrap(), DIMS
                        );
                    }
                    acc
                }
            }
        }
    }

    pub fn update_vel(&mut self) {
        for (vi, ai) in
        self.vel_vec.clone().iter_mut().zip(
            self.get_total_acc(&mut TREE_POINTER.lock().unwrap().clone())
        ) {
            *vi += ai*DT
        }
    }

    pub fn update_pos(&mut self) {
        for (pi, vi) in self.pos_vec.iter_mut().zip( self.vel_vec.clone() ) {
            *pi += vi*DT
        }
    }
}

impl Region {

    pub fn update_com(&mut self) {

        // we check whether we have child regions to determine whether
        // or not we're in a leaf node. If we are, we should just
        // update the com (assumign there is one), else we should
        // recurse into child regions and update those.

        match self.reg_vec {

            None => {

                // None means we're in a leaf node (or we have masses
                // waiting to be injected, which shouldn't happen)

                match self.com.clone() {

                    None => println!("superfluous call to update_com. change line 153 in physics.rs to panic! and use backtrace to see where."),

                    Some(_com) => {

                        match self.add_queue {
                            None => (),
                            Some(_) => panic!("cannot update com with masses waiting to be queued!"),
                        };

                        let mut com = self.com.clone().unwrap();
                        com.update_pos();
                        self.com = Some(com);

                    },
                }
            },

            // This assumes we've pruned dead children, which we
            // haven't quite done yet.

            Some(ref mut reg_vec) => {
                println!("I see dead children");
                let mut num = vec![0.0; DIMS as usize];
                let mut den = 0.0;

                for child in reg_vec.iter_mut() {
                    child.update_com();
                    match child.com {
                        None => continue,
                        Some(ref com) => {
                            let mut com = com.clone();
                            den += com.mass;
                            for i in 0..DIMS {
                                num[i] += com.pos_vec[i] * com.mass;
                            }
                        },
                    }
                }
                if den == 0.0 {
                    den = 1.0;
                }
                for i in 0..DIMS {
                    num[i] /= den
                }

                self.com = Some(Body {pos_vec: num, vel_vec: vec![0.0;
                    DIMS as usize], mass: den});
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dist_sq() {
        let m1 = Body {
            pos_vec: vec![1.0, 0.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0
        };

        let m2 = Body {
            pos_vec: vec![0.0, 0.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0
        };

        let m3 = Body {

            pos_vec: vec![-3.0, 0.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0
        };

        let m4 = Body {
            pos_vec: vec![0.0, 4.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0
        };

        assert_eq!(m1.squared_dist_to(&m2), 1.0);
        assert_eq!(m3.squared_dist_to(&m4), 25.0);
    }

    #[test]
    fn test_vec_rel() {
        let m1 = Body {
            pos_vec: vec![1.0; DIMS],
            vel_vec: vec![0.0; DIMS],
            mass: 0.0
        };

        let m2 = Body {
            pos_vec: vec![0.0; DIMS],
            vel_vec: vec![0.0; DIMS],
            mass: 0.0
        };

        // let m3 = Body {
        //     pos_vec: vec![-3.0; DIMS],
        //     vel_vec: vec![0.0; DIMS],
        //     mass: 0.0
        // };

        // let m4 = Body {
        //     pos_vec: vec![4.0].extend([0.0; DIMS-1].iter()),
        //     vel_vec: vec![0.0; DIMS],
        //     mass: 0.0
        // };
        // println!("m1 rel m2 {:?}", m1.vec_rel(&m2, DIMS));

        assert_eq!(m1.vec_rel(&m2, DIMS), vec![-1.0; DIMS]);
        // assert_eq!(m3.vec_rel(&m4), vec![7.0].extend(vec![0.0; DIMS-1]));
    }

    #[test]
    fn test_sq_mag() {
        let m1 = Body {
            pos_vec: vec![1.0, 0.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0
        };

        let m2 = Body {
            pos_vec: vec![0.0, 0.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0
        };

        let m3 = Body {
            pos_vec: vec![-3.0, 0.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0
        };

        let m4 = Body {
            pos_vec: vec![0.0, 4.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0
        };
        // println!("m1 rel m2 {:?}", m1.vec_rel(&m2));

        assert_eq!(m1.sq_magnitude(&m1.vec_rel(&m2, 3), 3), 1.0);
        assert_eq!(m3.sq_magnitude(&m3.vec_rel(&m4, 3), 3), 25.0);
    }

    #[test]
    fn test_is_far() {
        for dims in 1..9 {
            let x = (4.0/(dims as f64)).sqrt();

            let body = Body {
                pos_vec: vec![x; dims],
                vel_vec: vec![0.0; dims],
                mass: 0.0
            };

            let mut node = Region {

                reg_vec: None,
                coord_vec: vec![0.0; dims],
                half_length: 0.5,
                remove: false, // FIXME: remove?
                add_queue: None,
                com:
                Some(
                    Body {
                        pos_vec: vec![0.0; dims],
                        vel_vec: vec![0.0; dims],
                        mass: 0.0
                    }
                )

            };
            assert!(body.is_far(&mut node));
        }
    }

    #[test]
    fn test_get_classical_accel() {

        for dims in 1..9 {
            let body1 = Body {
                pos_vec: vec![1.0; dims],
                vel_vec: vec![0.0; dims],
                mass: 1.0
            };

            let body2 = Body {
                pos_vec: vec![0.0; dims],
                vel_vec: vec![0.0; dims],
                mass: 1.0
            };

            assert_eq!(
                body1.sq_magnitude(
                    &body1.get_classical_accel(&body2, dims), dims
                ).sqrt(),
                ( 6.674 / (1_000_000_000_00.0 * (dims as f64)) )
            );
        }
    }

    #[test]
    fn test_update_accel() {
        // past here, floating point error begins to add up.
        for dims in 1..5 {

            let body1 = Body {
                pos_vec: vec![1.0; dims],
                vel_vec: vec![0.0; dims],
                mass: 1.0
            };

            let body2 = Body {
                pos_vec: vec![0.0; dims],
                vel_vec: vec![0.0; dims],
                mass: 1.0
            };

            let acc = vec![0.0; dims];
            let entry = -1.0 * (6.674 / 1_000_000_000_00.0) / (dims as f64).sqrt() / (dims as f64);
            assert_eq!(body1.update_accel(acc, &body2, dims), vec![entry; dims]);

        }
    }

    #[test]
    fn test_get_total_acc() {

    }

    #[test]
    fn test_update_vel() {

    }

    #[test]
    fn test_update_pos() {

    }

    #[test]
    fn test_update_com() {

    }

}
