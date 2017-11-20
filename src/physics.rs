pub use super::tree::*;
pub use super::DIMS;
pub use super::TREE_POINTER;

static THETA: f64 = 0.5;
static DT: f64 = 0.01;

impl Body {
    fn squared_dist_to(&self, mass: &Body) -> f64 {
        let mut r_squared: f64 = 0.0;
        for (qi, pi) in self.pos_vec.iter().zip(&mass.pos_vec) {
            r_squared += (qi - pi).powi(2);
        }
        r_squared
    }


    fn is_far(&self, node: &mut Region) -> bool {
        // this makes me think we should store full-length instead of
        // half-length
        match node.com {
            None => {node.add_com(); self.is_far(node)},
            Some(ref com) => {
                (2.0 * node.half_length /
                    (self.squared_dist_to(&node.com.clone().unwrap()))
                    <= THETA) as bool
            },
        }
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

    fn update_accel(&self, mut acc: Vec<f64>, mass: &Body) -> Vec<f64> {
        for (mut acci, ai) in acc.iter_mut().zip(
            self.get_classical_accel(mass)) {
            *acci += ai;
        }
        acc
    }

    fn get_total_acc(&mut self, node: &mut Region) -> Vec<f64> {
        let mut acc = vec![0.0; DIMS];
        match node.reg_vec.clone() {
            None => self.update_accel(acc, &node.com.clone().unwrap()),
            Some(ref reg_vec) => {
                if self.is_far(node) {
                    self.update_accel(acc, &node.com.clone().unwrap())
                } else {
                    for child in reg_vec.iter() {
                        acc = self.update_accel(acc, &child.com.clone().unwrap());
                    }
                    acc
                }
            }
        }
    }

    fn update_vel(&mut self) {
        for (vi, ai) in
        self.vel_vec.iter_mut().zip(
            self.get_total_acc(TREE_POINTER)
        ) {
            *vi += ai*DT
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
        self.com = Some(self.calc_com());
    }
}
