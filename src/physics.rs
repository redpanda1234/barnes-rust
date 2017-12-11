// The physics module is really just a collection of methods on
// structs defined in tree that we've clustered here because they all
// have to do with the actual physics part of the simulation. So we
// have to move up one level (super), and import tree::*
pub use super::tree::*;
pub use super::data::*;

// Fetch global statics from the main function
pub use super::data::{DIMS, TREE_POINTER, DT, THETA};

// let const G: f64 = (6.674 / (1_000_000_000_00.0));
const G: f64 = 500.0;
use std::sync::{Arc, Mutex};

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
        // println!("called squared_dist_to");
        self.pos_vec
            .iter()
            .zip(&mass.pos_vec)
            .fold(0.0,(|sum,(qi, pi)| sum + (qi - pi).powi(2)))
    }

    pub fn node_sq_dist_to(&self, node: &Region) -> f64 {
        // println!("called node_sq_dist_to");
        // println!("woooo {:#?}, {:#?}", &node.coord_vec, self.pos_vec);
        self.pos_vec
            .iter()
            .zip(&node.coord_vec)
            .fold(0.0,(|sum,(qi, pi)| sum + (qi - pi).powi(2)))
    }

    // vec_rel gets the displacement vector between the calling mass
    // and some other passed Body.
    pub fn vec_rel(&self, mass: &Body) -> Vec<f64> {
        // println!("called vec_rel");
        self.pos_vec.iter()
            .zip(&mass.pos_vec)
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
        vec.iter().fold(0.0, |sum, vi| sum + vi.powi(2))
    }

    // is_far calculates a distance metric between the calling mass
    // (which really should only ever be the com of a leaf node in the
    // tree) and a passed region.

    pub fn is_far(&self, node_arc: Arc<Mutex<Region>>) -> bool {
        // println!("called is_far");
        // this makes me think we should store full-length instead of
        // half-length FIXME
        // FIXME: make sure this doesn't allow infinite loops;
        // i.e. that node.com will only be none if there's stuff
        // in the region_vec or add_queue.
        // println!("wheee {:#?}", self.node_sq_dist_to(&node));
        //Note: nodes are now guaranteed to have valid com when this is called
        let node = node_arc.try_lock().unwrap();
        ( 2.0 * node.half_length / self.node_sq_dist_to(&node).sqrt())
        // ( 2.0 * node.half_length / self.squared_dist_to(&node.com.clone().unwrap()))
            <= THETA
    }

    pub fn get_classical_accel(&self, mass: &Body) -> Vec<f64> {

        // this doesn't appear to work
        // if (self as *const _ == mass as *const _) {
        //     println!("\n\n\n FOUND SELF \n\n\n");
        // };

        //if the other body has no mass, just return 0
        if mass.mass == 0.0 {
            return vec![0.0; DIMS];
        }

        // println!("called get_classical_accel");
        let rel = self.vec_rel(mass);
        // println!("{:?}", rel);
        let sq_mag = self.sq_magnitude(&rel);
        // println!("{}, {:#?}", sq_mag, rel);
        let acc = mass.mass * G / sq_mag;
        // println!("{:?}", acc);
        let r = sq_mag.sqrt();

        //if the distance is small, just return 0
        //note that floats are weird, so the same mass
        //could have a nonzero distance to itself
        if r <= MIN_LEN / 100.0 {
            // println!("{:?}", r);
            return vec![0.0; DIMS];
        }

        let result = rel.iter()
                        .map(|ri| (ri/r) * acc)
                        .collect::<Vec<f64>>();
        //println!("acceleration: {:#?}", result);
        result
    }

    pub fn get_classical_potential(&self, mass: &Body) -> Vec<f64> {
        // use super::G;
        let rel = self.vec_rel(&mass);
        let sq_mag = self.sq_magnitude(&rel);
        // println!("{}, {:#?}", sq_mag, rel);
        let r = sq_mag.sqrt();
        let pot = mass.mass * G / r;

        //if the distance is 0, just return 0
        if(r == 0.0) {
            return vec![0.0; DIMS];
        }

        rel.iter()
            .map(|ri| ri * pot/r)
            .collect::<Vec<f64>>()
    }

    pub fn update_accel(&self, acc: Vec<f64>, mass_arc: Arc<Mutex<Body>>) -> Vec<f64> {
        // println!("called update_accel");
        let mass = mass_arc.try_lock().unwrap();
        acc.iter()
            .zip(self.get_classical_accel(&mass))
            .map(|(acc_self, acc_other)| acc_self + acc_other)
            .collect::<Vec<f64>>()
    }

    pub fn get_total_acc(&mut self, node_arc: Arc<Mutex<Region>>) -> Vec<f64> {
        println!("called get_total_acc");
        let mut acc = vec![0.0; DIMS];
        let mut match_me =
            node_arc
            .try_lock()
            .unwrap()
            .reg_vec.clone();
        match match_me {
            //if this is a leaf, find the acceleration between us and its com
            None => {
                // println!("matched None on first arm of get_totall_acc");
                // drop(match_me);
                // println!("try_locked node_arc and entered the match btry_lock. Matched on None");
                match node_arc.try_lock().unwrap().com {
                    None => {
                        // println!("matched None on subarm of None");
                        // println!("matched on None");
                        acc
                    },
                    Some(ref com_arc) => {
                        // println!("matched some on subarm of None");
                        let com = com_arc.try_lock().unwrap().clone();
                        let total_acc = self.update_accel(acc.clone(), Arc::new(Mutex::new(com)));
                        //println!("acceleration component: {:#?}", total_acc); // this is never called on singularities
                        // acc = acc.iter()
                        //     .zip(total_acc.iter())
                        //     .map(|(u,v)| u+v)
                        //     .collect::<Vec<f64>>();

                        total_acc
                    }
                }
            },
            //if this node has children, find the acceleration from each of them
            Some(_) => {
                // println!("matched Some on first arm of get_totall_acc");
                // println!("try_locked node_arc and entered the match btry_lock. Matched on Some");
                // println!("has reg_vec");
                let match_me_too = node_arc.try_lock().unwrap().com.clone();
                match match_me_too {

                    None => {
                        //we really shouldn't get here
                        node_arc.try_lock().unwrap().update_com();
                        self.get_total_acc(Arc::clone(&node_arc))
                    },

                    Some(ref com_arc) => {
                        // println!("matched Some on subarm of Some");
                        if self.is_far(Arc::clone(&node_arc)) {
                           // println!("was far");
                            let total_acc = self.update_accel(vec![0.0; DIMS], Arc::clone(com_arc));
                            //println!("acceleration component: {:#?}", total_acc);
                            // this is always 0 when stuff doesn't move, for some reason
                            // acc = acc
                            //     .iter()
                            //     .zip(total_acc.iter())
                            //     .map(|(u,v)| u+v).collect::<Vec<f64>>();
                            if com_arc.try_lock().unwrap().clone().mass > 0.0 {
                                //println!("{:#?}, {:#?}", total_acc, com_arc.try_lock().unwrap().clone());
                            }
                            total_acc
                        } else {
                            //println!("wasn't far from: {:#?}", com_arc.try_lock().unwrap().clone());
                            for mut child in match_me.unwrap().iter() {
                                let total_acc = self.get_total_acc(Arc::clone(child));
                                // println!("acceleration component: {:#?}", total_acc);
                                acc = acc.iter().zip(total_acc
                                        .iter()).map(|(u,v)| u+v).collect::<Vec<f64>>();
                                //println!("{:#?}: ", acc);
                            }
                            acc
                        }
                    }
                }
            }
        }
    }

    pub fn update_vel(&mut self) {
        // println!("called update_vel");
        //TODO: we shouldn't have to be cloning vel_vec, so let's find a better way
        //TODO: tree should be a reference so we don't have to copy it every time
        // println!("updating vel");
        // println!("old velocity component: {:#?}", self.vel_vec[0]);
        let mut tree = TREE_POINTER.try_lock().unwrap().tree.clone();
        // for child in tree.reg_vec.clone().unwrap() {
        //     let new_child = child.try_lock().unwrap().clone();
        //     // println!("{:#?}", child);
        //     let new_child = Arc::new(Mutex::new(new_child));
        //     let mut vel_vec = self.vel_vec.clone();

        //     let mut vel_vec = vel_vec.iter_mut().zip(
        //         self.get_total_acc(new_child))
        //     .map(|(vi, ai)| *vi + ai * DT).collect::<Vec<f64>>();
        //     self.vel_vec = vel_vec;
        // }

        println!("\n\ntrying to update acceleration...");

        self.vel_vec = tree.reg_vec.clone().unwrap().iter().fold(
                self.vel_vec.clone(), |vel, child| vel.iter().zip(
                    self.get_total_acc(child.clone())
                    ).map(|(vi, ai)| *vi + ai * DT).collect::<Vec<f64>>()
        )

        // println!("new velocity component: {:#?}", self.vel_vec[0]);
    }

    //TODO: make update_pos use functional programming
    pub fn update_pos(&mut self) {
        // println!("called update_pos");
        for (pi, vi) in self.pos_vec.iter_mut().zip( self.vel_vec.clone() ) {
            *pi += vi*DT;
        }

        // TODO update the normalized coordinates too

    }
}

impl Region {

    // Recursively update the accelerations and velocities of masses
    pub fn deep_update_vel(&mut self) {
        // println!("called deep_update_vel");
        match self.reg_vec.clone() {
            //if we're at the leaf node, call update_vel if we have a mass
            None => {
                // println!("matched None in first arm of update_vel");
                match self.com.clone() {
                    None => {
                        // println!("matched None on subarm");
                        ()
                    },
                    Some(com_arc) => {
                        // println!("matched Some on subarm");
                        let mut com_clone = com_arc.try_lock().unwrap().clone();
                        com_clone.update_vel();
                        self.com = Some(Arc::new(Mutex::new(com_clone)));
                        // drop(com_arc);
                        //TODO: find out if it's actually necessary to re-wrap this
                    }
                }
            },
            //if we have children, call recursively
            Some(ref mut reg_vec) => {
                // println!("matched Some in first arm of deep_update_vel");
                let mut temp: Vec<Arc<Mutex<Region>>> = Vec::new();
                for child in reg_vec {
                    let mut child_clone = child.try_lock().unwrap().clone();
                    child_clone.deep_update_vel();
                    temp.push(Arc::new(Mutex::new(child_clone)));
                    // drop(child);
                }
                self.reg_vec = Some(temp);
            }
        }
    }

    // Recursively update the positions of masses
    pub fn deep_update_pos(&mut self) {
        // println!("deep updating pos");
        match self.reg_vec.clone() {
            //if we're at the leaf node, call update_pos if we have a mass
            None => {
                // println!("matched None in first arm of deep_update_pos");
                match self.com.clone() {
                    None => {
                        // println!("matched None in subarm");
                        ()
                    },
                    Some(com) => {
                        // println!("matched Some in subarm");
                        // self.update_com();
                        com.try_lock().unwrap().update_pos();
                        // self.com = Some(com);
                        self.update_com();
                    }
                }
            },
            //if we have children, call recursively
            Some(ref mut reg_vec) => {
                let mut temp = vec![];
                for mut child in reg_vec {
                    child.try_lock().unwrap().deep_update_pos();
                    // temp.push(Arc::clone(&child));
                    temp.push(child.clone());
                }
                self.reg_vec = Some(temp);
                self.update_com();
            }
        };
    }

    pub fn update_com(&mut self) {
        //println!("called update_com");

        // we check whether we have child regions to determine whether
        // or not we're in a leaf node. If we are, we should just
        // update the com (assuming there is one)
        // otherwise, update based on the coms of the children

        match self.reg_vec {

            None => {

                // None means we're in a leaf node (or we have masses
                // waiting to be injected, which shouldn't happen).
                // Once we're confident we'll never call on an empty
                // add queue, we can probably unwrap straight away ---
                // although match operations _are_ cheap...

                match self.com.clone() {

                    None => println!("superfluous (?) call to update_com.
                        change this line in physics.rs to panic! and use backtrace to see where."),

                    Some(com_arc) => {

                        // Double check to make sure we don't have any
                        // masses waiting to be added to the region,
                        // as that'd mess up the com we calculate.

                        match self.add_queue {
                            None => (),
                            Some(_) => panic!("cannot update com with masses waiting to be queued!")
                        };

                        // check to see if this region still contains com
                        // if it doesn't, remove com and push it to the global tree
                        if !self.contains(Arc::clone(&com_arc)) {
                            //println!("push to global");
                            Region::push_body_global(Arc::clone(&com_arc));
                            self.com = None;
                        } // else {
                            // println!("does contain");
                        // }
                    },
                }
            },

            Some(ref mut reg_vec) => {
                // println!("I see dead children");
                let mut num = vec![0.0; DIMS as usize];
                let mut den = 0.0;

                for child in reg_vec.iter() {
                    let mut match_me = &child.try_lock().unwrap().com;
                    // println!("{:#?}", match_me);
                    match match_me {
                        &None => continue,
                        &Some(ref com_arc) => {
                            // drop(match_me);
                            let mut com = com_arc.try_lock().unwrap();
                            den += com.mass;
                            //TODO: we shouldn't have to be cloning pos_vec
                            num = num
                                .iter()
                                .zip(com.pos_vec.clone())
                                .map(|(pi, pv)| pi + pv * com.mass)
                                .collect::<Vec<f64>>();
                        },
                    }
                }
                //if we didn't add any masses, make sure we're not dividing by 0
                if den != 0.0 {
                    // println!("fix divide by 0");
                    num = num
                        .iter()
                        .map(|n| n / den)
                        .collect::<Vec<f64>>();
                    // println!("new num is {:#?}", num);
                } else {
                    num = self.coord_vec.clone()
                }

                self.com = Some(
                    Arc::new(Mutex::new(Body {
                    pos_vec: num,
                    vel_vec: vec![0.0; DIMS],
                    mass: den
                }))
                );
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

        assert_eq!(m1.vec_rel(&m2), vec![-1.0; DIMS]);
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
                mass: 0.0
            };

            let node = Arc::new(Mutex::new(Region {

                reg_vec: None,
                coord_vec: vec![0.0; dims],
                half_length: 0.5 * THETA,
                add_queue: None,
                com:
                Some(
                    Arc::new(Mutex::new(
                    Body {
                        pos_vec: vec![0.0; dims],
                        vel_vec: vec![0.0; dims],
                        mass: 0.0
                    }))
                )

            }));
            assert!(body.is_far(node));
        }
    }

    #[test]
    fn test_get_classical_accel() {

        for dims in 1..2 {
            let body1 = Body {
                pos_vec: vec![1.0; dims],
                vel_vec: vec![0.0; dims],
                mass: 1.0,
            };

            let body2 = Body {
                pos_vec: vec![0.0; dims],
                vel_vec: vec![0.0; dims],
                mass: 1.0
            };

            assert_eq!(
                body1.sq_magnitude(
                    &body1.get_classical_accel(&body2)).sqrt(),
                ( G / (dims as f64))
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

            let body2 = Arc::new(Mutex::new(Body {
                pos_vec: vec![0.0; dims],
                vel_vec: vec![0.0; dims],
                mass: 1.0
            }));

            let acc = vec![0.0; dims];
            let entry = -1.0 * (G) / (dims as f64).sqrt() / (dims as f64);
            assert_eq!(body1.update_accel(acc, body2), vec![entry; dims]);

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

mod analysis {
    use super::*;

    /*
    Function to get the distribution of the radii of particles
    in the simulation.
    This assumes a force center at the origin.
    It would be easy to
modify to give the distances from some other point,
    but this is probably unnecessary.
    */
    fn radial_distribution() {
        let tree = TREE_POINTER.try_lock().unwrap().tree.clone();
        let masses = tree.list_masses();
        let distances = masses.iter().map(|m| m.sq_magnitude(&m.pos_vec));

        //then we can print/graph the distance distributions

    }

    /*
    Finds the total (nonrelativistic) kinetic energy of particles.
    */
    fn kinetic_energy() {
        let mut tree = TREE_POINTER.try_lock().unwrap().tree.clone();
        let masses = tree.list_masses();
        let energies = masses.into_iter().map(|m|
                            0.5*m.mass * m.sq_magnitude(&m.vel_vec))
                            .collect::<Vec<f64>>();

        //we could do something with the energy distribution, but for now we'll
        //just print the total kinetic energy

        let total_energy = energies.iter()
                                .fold(0.0,|m,sum| m + sum);

    }

    /*
    Function to find the exact gravitational potential energy.
    Note that this doesn't make the same approximation as
    the acceleration calculations.
    TODO: add an option to calculate with this approximation, both for faster
        calculation and more consistent results
    */
    fn potential_energy() {
        let mut tree = TREE_POINTER.try_lock().unwrap().tree.clone();
        let masses = tree.list_masses();

        let potential_energies = masses.iter()
                                    .zip(masses.iter())
                                    .map(|(m1, m2)| m1.get_classical_potential(m2));
    }

}
