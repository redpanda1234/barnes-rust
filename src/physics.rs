pub use super::tree::*;
pub use super::DIMS;

static THETA: f64 = 0.5;


impl Body {
    fn squared_dist_to(&self, mass: &Body) -> f64 {
        let mut r_squared: f64 = 0.0;
        for (qi, pi) in self.pos_vec.iter().zip(&mass.pos_vec) {
            r_squared += (qi - pi).powi(2);
        }
        r_squared
    }

    fn get_classical_accel(&self, mass: &Body) -> Vec<f64> {
        let r_squared = self.squared_dist_to(mass);
        let r = r_squared.sqrt();

        let acc = mass.mass * (6.674 / (1_000_000_000_00.0))/r_squared;
        let mut vec = Vec::new();

        for i in 0..self.pos_vec.len() {
            // TODO: make this work for generic number of dimensions
            vec.push(self.pos_vec[i] * acc / r) // pos_vec[i]/r is trig
        }
        vec
    }

    fn update_accel(&self, acc: Vec<f64>, mass: &Body) -> Vec<f64> {
        for (acci, ai) in acc.iter().zip(
            self.get_classical_accel(mass)) {
            acci += ai;
        }
    }

    // This is bad. Want to update forces and velocities and positions
    // all separately. How to fix? Offload into 3 sep. functions?
    // fn euler_step(&mut self, mass: &Body, dt: f64) {
    //     let acc = self.get_classical_accel(mass);
    //     for (pi, vi, ai) in izip!(
    //         &mut self.pos_vec,
    //         &mut self.vel_vec,
    //         acc
    //     ) { // Note to confused future me: * dereferences things
    //         *vi += ai*dt;
    //         *pi += *vi*dt;
    //     }
    // }

    fn is_far(&self, node: &mut Region) -> bool {
        // this makes me think we should store full-length instead of
        // half-length
        match node.com {
            None => {node.add_com(); self.is_far(node)},
            Some(com) => {
                2.0 * node.half_length /
                    (self.squared_dist_to(&node.com.clone().unwrap()))
                    <= THETA
            },
        }
    }

    fn get_acc(&mut self, node: &mut Region) ->  {
        let acc = vec![0.0; DIMS];
        match node.reg_vec {
            None => self.update_accel(node.com),
            Some => {

            }
        }
        if self.is_far(node) {

        } else {

        }
    }
}

impl Region {
    fn calc_com(&self) -> Body {
        match self.reg_vec {
            None => self.com.clone().unwrap(),
            // This assumes we've pruned dead children, which we
            // haven't quite done yet.
            Some(ref reg_vec) => {
                let mut num = vec![0.0; DIMS as usize];
                let mut den = 0.0;

                for child in self.reg_vec.clone().unwrap().iter() {
                    den += child.calc_com().mass;
                    let com = child.com.clone().unwrap();
                    // vec = self.pos_vec.clone()
                    for i in 0..DIMS {
                        num[i] += com.pos_vec[i] * com.mass;
                    }
                }
                for i in 0..DIMS {
                    num[i] /= den
                }
                Body {pos_vec: num, vel_vec: vec![0.0; DIMS as usize],
                    mass: den}
            }
        }
    }

    fn add_com(&mut self) {
        self.com = self.calc_com();
    }
}
