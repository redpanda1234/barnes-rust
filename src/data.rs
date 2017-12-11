pub extern crate rand;

use super::*;

use std::sync::{Mutex, Arc};
use std::thread;

pub const DIMS: usize = 2;
pub const THETA: f64 = 0.0005;
pub const DT: f64 = 0.0007;

// approximate radius of the milky way
//pub const MAX_LEN: f64 = 500_000_000_000_000_000_000.0;

// approximate mass of R136a1 --- for obvious reasons, we probably
// shouldn't actually use this.
// pub const MAX_MASS: f64 =
// 62_635_700_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000_000.0;

pub const MAX_LEN: f64 = 1_000.0;
pub const MIN_LEN: f64 = 10.0;
pub const MAX_VEL: f64 = 10_000.0;
pub const MAX_MASS: f64 = 5000.0;

pub const GAMMA_SHAPE: f64 = 200.0; // these must all be positive
pub const GAMMA_SCALE: f64 = 500.0;
pub const GAMMA_SHAPE_TF: f64 = 200.0;
pub const GAMMA_SCALE_TF: f64 = 600.0;

pub const NORMAL_MEAN: f64 = 0.5 * MAX_LEN;
pub const NORMAL_STD_DEV: f64 = 0.5 * NORMAL_MEAN;

pub const NORMAL_MEAN_TF: f64 = 2.0 * NORMAL_MEAN;
pub const NORMAL_STD_DEV_TF: f64 = NORMAL_STD_DEV;

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

        vec[DIMS-2] = mag * 1.0*final_theta.sin() * product;
        vec[DIMS-1] = mag * 1.0*final_theta.cos() * product;

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
            mass: m + 1.0
        };

        Arc::new(Mutex::new(body))
    }

    // gt is for gen_tree
    pub fn gt_all_ranges(num_bodies: usize) {
        use data::rand::distributions::*;
        // let mut seeder = get_seeder_rng();

        let m_gen = Range::new(0.0, MAX_MASS);
        let p_mag_gen = Range::new(0.1*MAX_LEN, 0.5*MAX_LEN);
        let v_mag_gen = Range::new(0.4*MAX_VEL, 0.8*MAX_VEL);
        let t_gen = Range::new(0.0, PI);
        let t_f_gen = &Range::new(0.0, 2.0*PI);

        let mut rng = rand::StdRng::new().unwrap();

        for _ in 0..num_bodies {

            Region::push_body_global(
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

    pub fn gt_two_body() {
        Region::push_body_global(
            Arc::new(Mutex::new(
            Body {
                pos_vec: vec![-100.0, 0.0],
                vel_vec: vec![0.0, 2000.0],
                mass: 100000.0//m
            }
        )));
        Region::push_body_global(
            Arc::new(Mutex::new(
            Body {
                pos_vec: vec![100.0, 0.0],
                vel_vec: vec![0.0, -2000.0],
                mass: 100000.0//m
            }
        )));
    }

    //a system of two large objects, i.e. stars, with a number of
    //smaller objects injected around them
    pub fn gt_binary_system() {
        gt_two_body();
        
        gt_all_ranges(300);
    }

    //inject masses horizontally
    pub fn gt_scattering(num_bodies: usize) {
        use data::rand::distributions::*;

        //impact parameters
        let impact_parameters = Range::new(-400.0, 400.0);
        let mut rng = rand::StdRng::new().unwrap();

        let velocities = Range::new(750.0, 10000.0);
        let offsets = Range::new(50.0, 150.0);

        for _ in 0..num_bodies {

            let b = impact_parameters.ind_sample(&mut rng);
            let v = velocities.ind_sample(&mut rng);
            let x = offsets.ind_sample(&mut rng);       

            Region::push_body_global(
                Arc::new(Mutex::new(
                Body {
                    pos_vec: vec![-MAX_LEN + x, b],
                    vel_vec: vec![v, 0.0],
                    mass: 1.0//m
                }
            )));
        }
    }

    //scattering in a 1/r potential
    pub fn gt_rutherford_scattering(num_bodies: usize) {

        Region::push_body_global(
            Arc::new(Mutex::new(
            Body {
                pos_vec: vec![0.0, 0.0],
                vel_vec: vec![0.0, 0.0],
                mass: 100000.0//m
            }
        )));

        gt_scattering(num_bodies);
    }

    //scattering onto a binary system
    pub fn gt_binary_scattering(num_bodies: usize) {
        gt_two_body();

        gt_scattering(num_bodies);
    }


    
    pub fn gt_all_gamma(num_bodies: usize) {
        use data::rand::distributions::*;
        // let mut seeder = get_seeder_rng();

        let m_gen = Gamma::new(GAMMA_SHAPE, GAMMA_SCALE);
        let p_mag_gen = Gamma::new(GAMMA_SHAPE, GAMMA_SCALE);
        let v_mag_gen = Gamma::new(GAMMA_SHAPE, GAMMA_SCALE);
        let t_gen = Gamma::new(GAMMA_SHAPE, GAMMA_SCALE);
        let t_f_gen = &Gamma::new(GAMMA_SHAPE_TF, GAMMA_SCALE_TF);

        let mut rng = rand::StdRng::new().unwrap();

        for _ in 0..num_bodies {

            Region::push_body_global(
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

        Region::push_body_global(
            Arc::new(Mutex::new(
            Body {
                pos_vec: vec![-50.0; DIMS],
                vel_vec: vec![0.0; DIMS],
                mass: 10000.0//m
            }
        )));
    }

    pub fn gt_all_normal(num_bodies: usize) {
        use data::rand::distributions::*;
        // let mut seeder = get_seeder_rng();

        let m_gen = Normal::new(NORMAL_MEAN, NORMAL_STD_DEV);
        let p_mag_gen = Normal::new(NORMAL_MEAN, NORMAL_STD_DEV);
        let v_mag_gen = Normal::new(NORMAL_MEAN, NORMAL_STD_DEV);
        let t_gen = Normal::new(NORMAL_MEAN, NORMAL_STD_DEV);
        let t_f_gen = Normal::new(NORMAL_MEAN_TF, NORMAL_STD_DEV_TF);

        let mut rng = rand::StdRng::new().unwrap();

        for _ in 0..num_bodies {

            Region::push_body_global(
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

    // fn push_body_global(body_arc: Arc<Mutex<Body>>) {
    //     let match_me = TREE_POINTER.try_lock().unwrap().tree.add_queue.clone();
    //     match match_me {

    //         None => {
    //             let mut add_me  = Vec::new();
    //             add_me.push(body_arc);

    //             TREE_POINTER.try_lock().unwrap().tree.add_queue = Some(add_me);
    //         },

    //         Some(_) => {
    //             let mut queue =
    //                 TREE_POINTER.try_lock().unwrap().tree.add_queue.clone().unwrap();
    //             queue.push(body_arc);
    //             TREE_POINTER.try_lock().unwrap().tree.add_queue = Some(queue);
    //         }
    //     }
    // }
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
