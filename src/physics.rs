pub use super::tree::*;

impl Body {
    fn get_classical_accel(&self, mass: &Body) -> Vec<f64> {
        let mut r_squared: f64 = 0.0;
        for (qi, pi) in self.pos_vec.iter().zip(&mass.pos_vec) {
            r_squared += (qi - pi).powi(2);
        }
        let r = r_squared.sqrt();
        let frc = mass.mass * (6.674 / (1_000_000_000_00.0))/r_squared;
        let mut vec = Vec::new();
        let pos_vec = self.pos_vec.clone();
        for i in 0..pos_vec.len() {
            vec.push(self.pos_vec[i] * frc / r)
        }
        vec
    }

    fn euler_step(&mut self, mass: &Body, dt: f64) {
        let acc = self.get_classical_accel(mass);
        for (pi, vi, ai) in izip!(
            &mut self.pos_vec,
            &mut self.vel_vec,
            acc
        ) { // Note to confused future me: * dereferences
            *vi += ai*dt;
            *pi += *vi*dt;
        }
    }
}
