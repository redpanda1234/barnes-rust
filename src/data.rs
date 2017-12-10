pub extern crate rand;

use super::*;

use std::sync::{Mutex, Arc};
use std::thread;

// TODO: use this everywhere we check dimensions
pub const DIMS: usize = 2;
pub const THETA: f64 = 0.5;
pub const DT: f64 = 10.0;

// approximate radius of the milky way
//pub const MAX_LEN: f64 = 500_000_000_000_000_000_000.0;

// approximate mass of R136a1 --- for obvious reasons, we probably
// shouldn't actually use this.
// pub const MAX_MASS: f64 =
// 62_635_700_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000.0;

pub const MAX_LEN: f64 = 1_00.0;
pub const MAX_VEL: f64 = 0.01;
pub const MAX_MASS: f64 = 1_000.0;
pub static mut NUM_THREADS: i64 = 20;

pub struct TreeWrapper {
    pub tree: Region
}

// TODO: make our organization here more intelligent. Should probably
// offload most statics  to their own dedicated module, along with
// static generation. Maybe data.rs?
pub mod gen_mult {
    pub fn populate_mult(n: usize, mult: f64) -> Vec<Vec<f64>> {
        if n <= 0 {
            return vec![vec![mult]];
        }

        let mut v1: Vec<Vec<f64>> = populate_mult(n - 1, -1.0);
        v1.extend(populate_mult(n - 1, 1.0));

        if mult != 0.0 {
            for i in 0..v1.len() {
                v1[i].push(mult);
            }
        }

        v1
    }
}

pub mod generate {

    use data::rand::*;
    use data::rand::distributions::{IndependentSample};
    use data::*;
    use tree::*;

    use std::f64::consts::PI;

    // Using a pseudo-randomly-generated scalar value for the
    // magnitude of our output vector, this function uses n-d
    // spherical coordinates to transform back into a r_vec in the
    // standard basis.

    // Here, T is a generic type representing an arbitrary rng
    // generator or theta. We made this generic so that we can look at
    // different distributions of thetas, e.g. gamma, normal, etc. rng
    // is the seeded rng generator we'll be using to generate our
    // values. The additional T1 is because the last angle needs to be
    // generated over a different range, and we don't want to write
    // something that matches on types of T and construct a new
    // generator just for a single value.

    // T should range from 0 to pi, T1 from 0 to 2pi.

    // See HERE
    //
    // https://en.wikipedia.org/wiki/N-sphere#Spherical_coordinates
    //
    // for an explanation on what this function is _supposed_ to be
    // doing

    pub fn nd_vec_from_mag<T: IndependentSample<f64>>(
        mag: f64,
        t_generator: &T,
        final_theta: f64,
        mut rng: StdRng
    ) -> Vec<f64> {

        let mut vec = vec![0.0; DIMS];

        // The final case is special, so we don't iterate all the way
        // through DIMS.

        // This'll hold the running product of sin values of each of
        // the thetas defining our position

        let mut product: f64 = 1.0;

        for i in 0..(DIMS-2) {
            let theta = t_generator.ind_sample(&mut rng);
            vec[i] = mag*(theta.cos())*product;

            // all future calculations will involve product of
            // preceding theta.sin() values, so we increment it here

            product *= theta.sin();

        }

        // The final theta value is special, as it ranges from 0 to
        // 2pi. So we treat it the r coordinates whose definitions
        // involve it in special cases outside of our loop. Note that
        // the final r_vec entry involves just .sin()'s, no .cos()'s.

        vec[DIMS-2] = mag * final_theta.cos() * product;
        vec[DIMS-1] = mag * final_theta.sin() * product;

        // return vec
        vec
    }

    pub fn gb_from_mags<T: IndependentSample<f64>>(
        t_f1: f64,
        t_f2: f64,
        p_mag: f64,
        v_mag: f64,
        m: f64,
        t_generator: T,
    ) -> Arc<Mutex<Body>> {
        let mut rng1 = rand::StdRng::new().unwrap();
        let mut rng2 = rand::StdRng::new().unwrap();

        let pos = nd_vec_from_mag(p_mag, &t_generator, t_f1, rng1);
        let vel = nd_vec_from_mag(v_mag, &t_generator, t_f2, rng2);

        let body = Body {
            pos_vec: pos,
            vel_vec: vel,
            mass: m
        };

        Arc::new(Mutex::new(body))
    }

    // gt is for gen_tree
    pub fn gt_all_ranges(num_bodies: usize) {
        use data::rand::distributions::*;
        // let mut seeder = get_seeder_rng();

        let m_gen = Range::new(0.0, MAX_MASS);
        let p_mag_gen = Range::new(0.0, MAX_LEN);
        let v_mag_gen = Range::new(0.0, MAX_VEL);
        let t_gen = Range::new(0.0, PI);
        let t_f_gen = &Range::new(0.0, 2.0*PI);

        let mut rng = rand::StdRng::new().unwrap();

        for _ in 0..num_bodies {

            push_body_global(
                gb_from_mags(
                    t_f_gen.ind_sample(&mut rng),
                    t_f_gen.ind_sample(&mut rng),
                    p_mag_gen.ind_sample(&mut rng),
                    v_mag_gen.ind_sample(&mut rng),
                    m_gen.ind_sample(&mut rng),
                    t_gen
                )
            )
        }
    }

    fn push_body_global(body_arc: Arc<Mutex<Body>>) {
        let match_me = TREE_POINTER.try_lock().unwrap().tree.add_queue.clone();
        match match_me {

            None => {
                let mut add_me  = Vec::new();
                add_me.push(body_arc);

                TREE_POINTER.try_lock().unwrap().tree.add_queue = Some(add_me);
            },

            Some(_) => {
                let mut queue =
                    TREE_POINTER.try_lock().unwrap().tree.add_queue.clone().unwrap();
                queue.push(body_arc);
                TREE_POINTER.try_lock().unwrap().tree.add_queue = Some(queue);
            }
        }
    }
}

lazy_static! {

    // I know, I know... Making a global variable is bad enough, but
    // _this_... I mean, why not just construct it in main, and pass a
    // reference to all functions that need it?? Answer: refactoring
    // is a pain. Will fix later? Maybe with our data-generation
    // scheme this might be ideal.

    // TOFIX: redefine TREE_POINTER such that we can access the global
    // region_vector without try_locking the Region itself. This will
    // allow us to handle the add_queue and reg_vec separately, which
    // will improve computation times.


    //Stores a TreeWrapper that holds the global tree
    pub static ref TREE_POINTER: Arc<Mutex<TreeWrapper>> =
        Arc::new(
            Mutex::new(
                TreeWrapper {
                    tree: Region {
                        reg_vec: None,
                        coord_vec: vec![0.0; DIMS],
                        half_length: MAX_LEN,
                        add_queue: Some(Vec::new()),
                        com: None
                    }
                }
            )
        );

    /*
    // MULTIPLIERS is a static array that we'll use later to quickly
    // determine the centers of subregions when we recurse. If we
    // multiply each of the sub-arrays in MULTIPLIERS by the
    // sidelength of our region, then _add_ those to our position
    // vector for our starting region, it'll get us the center of our
    // new region.
    */

    pub static ref MULTIPLIERS: Mutex<Vec<Vec<f64>>> = Mutex::new(
        gen_mult::populate_mult(DIMS, 0.0)
    );
}
