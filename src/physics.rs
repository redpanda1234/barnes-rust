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
        self.pos_vec.iter().zip(&mass.pos_vec)
            .fold(0.0,(|sum,(qi, pi)| sum + (qi - pi).powi(2)))
    }

    // vec_rel gets the displacement vector between the calling mass
    // and some other passed Body.
    pub fn vec_rel(&self, mass: &Body) -> Vec<f64> {
        self.pos_vec.iter().zip(&mass.pos_vec)
            .map(|(pi, mi)| mi - pi)
                .collect::<Vec<f64>>()
    }

    // sq_magnitude should really probably just be its own function,
    // there's really no need to define it as a method here. TODO: fix
    // this, by moving it to a separate module? It takes as input some
    // vector, and finds the square of its magnitude. This is helpful
    // to define separately from squared_dist_to, even though they're
    // functionally equivalent (but this is slower), because we don't
    // always need to find the displacement vector.

    pub fn sq_magnitude(&self, vec: &Vec<f64>) -> f64 {
        vec.iter().fold(0.0,|sum,vi| sum + vi.powi(2))
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

    pub fn get_classical_accel(&self, mass: &Body) -> Vec<f64> {
        let mut rel = self.vec_rel(mass);
        let sq_mag = self.sq_magnitude(&rel);
        // println!("{}, {:#?}", sq_mag, rel);
        let acc = mass.mass * (6.674 / (1_000_000_000_00.0)) / sq_mag;
        let r = sq_mag.sqrt();

        rel.iter().map(|ri| ri * acc/r).collect::<Vec<f64>>()
    }

    pub fn update_accel(&self, mut acc: Vec<f64>, mass: &Body) -> Vec<f64> {
        acc.iter().zip(self.get_classical_accel(mass))
            .map(|(accSelf, accOther)| accSelf + accOther).collect::<Vec<f64>>()
    }

    pub fn get_total_acc(&mut self, node: &mut Region) -> Vec<f64> {
        let mut acc = vec![0.0; node.reg_vec.iter().len()];
        match node.reg_vec.clone() {
            None => self.update_accel(acc, &node.com.clone().unwrap()),
            Some(ref reg_vec) => {
                if self.is_far(node) {
                    self.update_accel(acc, &node.com.clone().unwrap())
                } else {
                    for child in reg_vec.iter() {
                        acc = self.update_accel(
                            acc, &child.com.clone().unwrap());
                    }
                    acc
                }
            }
        }
    }

    pub fn update_vel(&mut self) {
        //TODO: we shouldn't have to be cloning vel_vec, so let's find a better way
        self.vel_vec = self.vel_vec.clone().iter_mut().zip(
            self.get_total_acc(&mut TREE_POINTER.lock().unwrap().clone()))
            .map(|(vi, ai)| *vi + ai * DT).collect::<Vec<f64>>();
    }

    //TODO: make update_pos use function programming
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
        // update the com (assuming there is one), else we should
        // recurse into child regions and update those.

        match self.reg_vec {

            None => {

                // None means we're in a leaf node (or we have masses
                // waiting to be injected, which shouldn't happen).
                // Once we're confident we'll never call on an empty
                // add queue, we can probably unwrap straight away ---
                // although match operations _are_ cheap...

                match self.com.clone() {

                    None => println!("superfluous (?) call to update_com. change line 153 in physics.rs to panic! and use backtrace to see where."),

                    Some(mut com) => {

                        // Double check to make sure we don't have any
                        // masses waiting to be added to the region,
                        // as that'd mess up the com we calculate.

                        match self.add_queue {
                            None => (),
                            Some(_) => panic!("cannot update com with masses waiting to be queued!"),
                        };

                        com.update_vel();
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
                            //TODO: we shouldn't have to be cloning pos_vec
                            num = num.iter().zip(com.pos_vec.clone()).map(|(pi, pv)| pi + pv * com.mass)
                                .collect::<Vec<f64>>();
                        },
                    }
                }
                //if we didn't add any masses, make sure we're not dividing by 0
                if den == 0.0 {
                    den = 1.0;
                }
<<<<<<< HEAD
                
                num = num.iter().map(|n| n / den).collect::<Vec<f64>>();

=======
                for i in 0..DIMS {
                    num[i] /= den
                }
                let node_id = String::from("o");
>>>>>>> c9b9cfcbd9b948ebc97f1aaabd4d7c42a8c1940b
                self.com = Some(Body {pos_vec: num, vel_vec: vec![0.0;
                    DIMS as usize], mass: den, id: node_id});
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
            mass: 0.0,
            id: String::from("m1")
        };

        let m2 = Body {
            pos_vec: vec![0.0, 0.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0,
            id: String::from("m2")
        };

        let m3 = Body {

            pos_vec: vec![-3.0, 0.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0,
            id: String::from("m3")
        };

        let m4 = Body {
            pos_vec: vec![0.0, 4.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0,
            id: String::from("m4")
        };

        assert_eq!(m1.squared_dist_to(&m2), 1.0);
        assert_eq!(m3.squared_dist_to(&m4), 25.0);
    }

    #[test]
    fn test_vec_rel() {
        let m1 = Body {
            pos_vec: vec![1.0; DIMS],
            vel_vec: vec![0.0; DIMS],
            mass: 0.0,
            id: String::from("m1")
        };

        let m2 = Body {
            pos_vec: vec![0.0; DIMS],
            vel_vec: vec![0.0; DIMS],
            mass: 0.0,
            id: String::from("m1")
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

        assert_eq!(m1.vec_rel(&m2), vec![-1.0; DIMS]);
        // assert_eq!(m3.vec_rel(&m4), vec![7.0].extend(vec![0.0; DIMS-1]));
    }

    #[test]
    fn test_sq_mag() {
        let m1 = Body {
            pos_vec: vec![1.0, 0.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0,
            id: String::from("m1")
        };

        let m2 = Body {
            pos_vec: vec![0.0, 0.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0,
            id: String::from("m1")
        };

        let m3 = Body {
            pos_vec: vec![-3.0, 0.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0,
            id: String::from("m1")
        };

        let m4 = Body {
            pos_vec: vec![0.0, 4.0, 0.0],
            vel_vec: vec![0.0, 0.0, 0.0],
            mass: 0.0,
            id: String::from("m1")
        };
        // println!("m1 rel m2 {:?}", m1.vec_rel(&m2));

        assert_eq!(m1.sq_magnitude(&m1.vec_rel(&m2)), 1.0);
        assert_eq!(m3.sq_magnitude(&m3.vec_rel(&m4)), 25.0);
    }

    #[test]
    fn test_is_far() {
        for dims in 1..9 {
            let x = (4.0/(dims as f64)).sqrt();

            let body = Body {
                pos_vec: vec![x; dims],
                vel_vec: vec![0.0; dims],
                mass: 0.0,
                id: String::from("m1")
            };

            let mut node = Region {

                reg_vec: None,
                coord_vec: vec![0.0; dims],
                half_length: 0.5,
                add_queue: None,
                com:
                Some(
                    Body {
                        pos_vec: vec![0.0; dims],
                        vel_vec: vec![0.0; dims],
                        mass: 0.0,
                        id: String::from("m1")
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
                mass: 1.0,
                id: String::from("m1")
            };

            let body2 = Body {
                pos_vec: vec![0.0; dims],
                vel_vec: vec![0.0; dims],
                mass: 1.0,
                id: String::from("m1")
            };

            assert_eq!(
                body1.sq_magnitude(
                    &body1.get_classical_accel(&body2)).sqrt(),
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
                mass: 1.0,
                id: String::from("m1")
            };

            let body2 = Body {
                pos_vec: vec![0.0; dims],
                vel_vec: vec![0.0; dims],
                mass: 1.0,
                id: String::from("m1")
            };

            let acc = vec![0.0; dims];
            let entry = -1.0 * (6.674 / 1_000_000_000_00.0) / (dims as f64).sqrt() / (dims as f64);
            assert_eq!(body1.update_accel(acc, &body2), vec![entry; dims]);

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
