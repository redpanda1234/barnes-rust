pub use super::tree::*;

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

    fn euler_step(&mut self, mass: &Body, dt: f64) {
        let acc = self.get_classical_accel(mass);
        for (pi, vi, ai) in izip!(
            &mut self.pos_vec,
            &mut self.vel_vec,
            acc
        ) { // Note to confused future me: * dereferences things
            *vi += ai*dt;
            *pi += *vi*dt;
        }
    }

    fn is_far(&self, node: &mut Region) -> bool {
        // this makes me think we should store full-length instead of
        // half-length
        match node.com {
            None => {node.get_com(); self.is_far(node)}
            Some(com) => {
                2.0 * node.half_length /
                    (self.squared_dist_to(&node.com.clone().unwrap()))
                    <= THETA
            }
        }
    }
}
