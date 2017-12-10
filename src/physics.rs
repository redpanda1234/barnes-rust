// The physics module is really just a collection of methods on
// structs defined in tree that we've clustered here because they all
// have to do with the actual physics part of the simulation. So we
// have to move up one level (super), and import tree::*
pub use super::tree::*;

// Fetch global statics from the main function
pub use super::data::{DIMS, TREE_POINTER, DT, THETA};

// let const G: f64 = (6.674 / (1_000_000_000_00.0));
const G: f64 = 100.0;

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
        self.pos_vec.iter().zip(&mass.pos_vec)
            .fold(0.0,(|sum,(qi, pi)| sum + (qi - pi).powi(2)))
    }

    pub fn node_sq_dist_to(&self, node: &Region) -> f64 {
        // println!("woooo {:#?}, {:#?}", &node.coord_vec, self.pos_vec);
        self.pos_vec.iter().zip(&node.coord_vec)
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
        vec.iter().fold(0.0, |sum, vi| sum + vi.powi(2))
    }

    // is_far calculates a distance metric between the calling mass
    // (which really should only ever be the com of a leaf node in the
    // tree) and a passed region.

    pub fn is_far(&self, node_arc: Arc<Mutex<Region>>) -> bool {
        // this makes me think we should store full-length instead of
        // half-length FIXME
        // FIXME: make sure this doesn't allow infinite loops;
        // i.e. that node.com will only be none if there's stuff
        // in the region_vec or add_queue.
        // println!("wheee {:#?}", self.node_sq_dist_to(&node));
        //Note: nodes are now guaranteed to have valid com when this is called
        let node = node_arc.lock().unwrap();
        ( 2.0 * node.half_length / self.node_sq_dist_to(&node))
        // ( 2.0 * node.half_length / self.squared_dist_to(&node.com.clone().unwrap()))
            <= THETA
    }

    pub fn get_classical_accel(&self, mass: &Body) -> Vec<f64> {
        let rel = self.vec_rel(mass);
        // println!("{:?}", rel);
        let sq_mag = self.sq_magnitude(&rel);
        // println!("{}, {:#?}", sq_mag, rel);
        let acc = mass.mass * G / sq_mag;
        // println!("{:?}", acc);
        let r = sq_mag.sqrt();

        //if the distance is 0, just return 0
        if r == 0.0 {
            // println!("{:?}", r);
            return vec![0.0; DIMS];
        }

        rel.iter().map(|ri| (ri/r) * acc).collect::<Vec<f64>>()
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

        rel.iter().map(|ri| ri * pot/r).collect::<Vec<f64>>()
    }

    pub fn update_accel(&self, acc: Vec<f64>, mass_arc: Arc<Mutex<Body>>) -> Vec<f64> {
        let mass = mass_arc.lock().unwrap();
        acc.iter().zip(self.get_classical_accel(&mass))
            .map(|(acc_self, acc_other)| acc_self + acc_other).collect::<Vec<f64>>()
    }

    pub fn get_total_acc(&mut self, node_arc: Arc<Mutex<Region>>) -> Vec<f64> {
        let mut acc = vec![0.0; DIMS];
        match node_arc.lock().unwrap().reg_vec {
            //if this is a leaf, find the acceleration between us and its com
            None => {
                match node_arc.lock().unwrap().com {
                    None => //{println!("node has no com"); acc},
                        acc,
                    Some(com_arc) => {
                        let total_acc = self.update_accel(acc.clone(), com_arc);
                        // println!("acceleration component: {:#?}", total_acc);
                        acc = acc.iter().zip(total_acc
                            .iter()).map(|(u,v)| u+v).collect::<Vec<f64>>();
                        acc
                    }
                }
            }
            //if this node has children, find the acceleration from each of them
            Some(ref mut reg_vec) => {
                // println!("has reg_vec");
                match node_arc.lock().unwrap().com {
                    None => {
                        // println!("updating child com");
                        node_arc.lock().unwrap().update_com();
                        self.get_total_acc(node_arc)
                    }
                    Some(com) => {
                        if self.is_far(node_arc) {
                            // println!("{:#?}, {:#?}", acc.clone(), com);
                            let total_acc = self.update_accel(acc.clone(), com);
                            // println!("acceleration component: {:#?}", total_acc);
                            acc = acc.iter().zip(total_acc
                                                 .iter()).map(|(u,v)| u+v).collect::<Vec<f64>>();
                            acc
                        } else {
                            for mut child in reg_vec.iter() {
                                let total_acc = self.get_total_acc(*child);
                                // println!("acceleration component: {:#?}", total_acc);
                                acc = acc.iter().zip(total_acc
                                        .iter()).map(|(u,v)| u+v).collect::<Vec<f64>>();
                            }
                            acc
                        }
                    }
                }
            }
        }
    }

    pub fn update_vel(&mut self) {
        // println!("aaaaa");
        //TODO: we shouldn't have to be cloning vel_vec, so let's find a better way
        //TODO: tree should be a reference so we don't have to copy it every time
        // println!("updating vel");
        // println!("old velocity component: {:#?}", self.vel_vec[0]);
        let mut tree = TREE_POINTER.lock().unwrap().tree.clone();
        for child in tree.reg_vec.iter_mut() {
            self.vel_vec = self.clone().vel_vec.iter_mut().zip(
            self.clone().get_total_acc(child[0]))
            .map(|(vi, ai)| *vi + ai * DT).collect::<Vec<f64>>();
        }

        // println!("new velocity component: {:#?}", self.vel_vec[0]);
    }

    //TODO: make update_pos use functional programming
    pub fn update_pos(&mut self) {
        // println!("updating pos");
        for (pi, vi) in self.pos_vec.iter_mut().zip( self.vel_vec.clone() ) {
            *pi += vi*DT;
        }

        // TODO update the normalized coordinates too

    }
}

impl Region {

    // Recursively update the accelerations and velocities of masses
    pub fn deep_update_vel(&mut self) {
        // println!("deep updating vel");
        match self.reg_vec.clone() {
            //if we're at the leaf node, call update_vel if we have a mass
            None => {
                match self.com.clone() {
                    None => (),
                    Some(com_arc) => {
                        com_arc.lock().unwrap().update_vel();
                        //TODO: find out if it's actually necessary to re-wrap this

                    }
                }
            },
            //if we have children, call recursively
            Some(ref mut reg_vec) => {
                let mut temp = vec![];
                for mut child in reg_vec {
                    child.lock().unwrap().deep_update_vel();
                    temp.push(child.clone());
                }
                self.reg_vec = Some(temp);
            }
        }
    }

    // Recursively update the postions of masses
    pub fn deep_update_pos(&mut self) {
        // println!("deep updating pos");
        match self.reg_vec.clone() {
            //if we're at the leaf node, call update_pos if we have a mass
            None => {
                match self.com.clone() {
                    None => (),
                    Some(com) => {
                        com.lock().unwrap().update_pos();

                    }
                }
            },
            //if we have children, call recursively
            Some(ref mut reg_vec) => {
                let mut temp = vec![];
                for mut child in reg_vec {
                    child.lock().unwrap().deep_update_pos();
                    temp.push(child.clone());
                }
                self.reg_vec = Some(temp);
                self.update_com();
            }
        };
    }

    pub fn update_com(&mut self) {
        // println!("called update_com");

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

                    Some(mut com) => {

                        // Double check to make sure we don't have any
                        // masses waiting to be added to the region,
                        // as that'd mess up the com we calculate.

                        match self.add_queue {
                            None => (),
                            Some(_) => panic!("cannot update com with masses waiting to be queued!"),
                        };
                        self.com = Some(com);
                    },
                }
            },

            Some(ref mut reg_vec) => {
                // println!("I see dead children");
                let mut num = vec![0.0; DIMS as usize];
                let mut den = 0.0;

                for child in reg_vec.iter_mut() {
                    match child.lock().unwrap().com {
                        None => continue,
                        Some(com_arc) => {
                            let mut com = com_arc.lock().unwrap();
                            den += com.mass;
                            //TODO: we shouldn't have to be cloning pos_vec
                            num = num.iter().zip(com.pos_vec.clone()).map(|(pi, pv)| pi + pv * com.mass)
                                .collect::<Vec<f64>>();
                        },
                    }
                }
                //if we didn't add any masses, make sure we're not dividing by 0
                if den != 0.0 {
                    // println!("fix divide by 0");
                    num = num.iter().map(|n| n / den).collect::<Vec<f64>>();
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

            let body2 = Body {
                pos_vec: vec![0.0; dims],
                vel_vec: vec![0.0; dims],
                mass: 1.0
            };

            let acc = vec![0.0; dims];
            let entry = -1.0 * (G) / (dims as f64).sqrt() / (dims as f64);
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
        let tree = TREE_POINTER.lock().unwrap().tree.clone();
        let masses = tree.list_masses();
        let distances = masses.iter().map(|m| m.sq_magnitude(&m.pos_vec));

        //then we can print/graph the distance distributions

    }

    /*
    Finds the total (nonrelativistic) kinetic energy of particles.
    */
    fn kinetic_energy() {
        let mut tree = TREE_POINTER.lock().unwrap().tree.clone();
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
        let mut tree = TREE_POINTER.lock().unwrap().tree.clone();
        let masses = tree.list_masses();

        let potential_energies = masses.iter()
                                    .zip(masses.iter())
                                    .map(|(m1, m2)| m1.get_classical_potential(m2));
    }

}
