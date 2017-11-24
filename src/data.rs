extern crate rand;

use super::*;

// TODO: use this everywhere we check dimensions
pub const DIMS: usize = 3;
pub const THETA: f64 = 0.5;
pub const DT: f64 = 0.01;
pub const MAX_LEN: f64 = 100.0;
pub static mut NUM_THREADS: i64 = 20;

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
    use data::{DIMS};
    use tree::*;

    // Returns the initial RNG-boi we'll be using to generate our
    // other RNG instances
    fn get_seeder_rng() -> StdRng {
        let seed: &[_] = &[1, 2, 3, 4];
        SeedableRng::from_seed(seed)
    }

    // When testing, we'll want to run the same simulations
    // repeatedly, to make sure we're actually modifying how the
    // system behaves. get_rng is used to ensure that we'll be
    // seeding all random elements of the simulation with the same
    // pseudorandom conditions each time.

    // maybe this isn't necessary, but I wanted each of the random
    // generators to be using different rng iterators from each other?
    // FIXME: determine wtf to do. Probably need this method for when
    // we multithread and can't  have multiple threads calling the
    // same rng object at the same time.

    // seed should be generated with get_seeder_rng(). Currently not
    // sure how to make this work, FIXME

    // fn get_rng<T>(seed: Seed) -> StdRng {
    //     SeedableRng::from_seed(seed)
    // }

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

    pub fn nd_vec_from_mag<T>(
        mag: f64,
        t_generator: T,
        final_theta: f64,
        rng: StdRng
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

    // gb is for gen_body
    // a generic body generator that takes a generic random number
    // generator for obtaining thetas.

    pub fn gb_from_mags<T: IndependentSample>(
        t_f: f64,
        p_mag: f64,
        v_mag: f64,
        m: f64,
        t_generator: T,
        seeder: StdRng
    ) -> Body {
        Body {
            pos_vec: nd_vec_from_mag(p_mag, t_generator, t_f, seeder),
            vel_vec: nd_vec_from_mag(v_mag, t_generator, t_f, seeder),
            mass: m
        }
    }

    // gt is for gen_tree
    // pub fn gt_ranges(num_bodies: usize) -> Region {

    // }

}

lazy_static! {

    // I know, I know... Making a global variable is bad enough, but
    // _this_... I mean, why not just construct it in main, and pass a
    // reference to all functions that need it?? Answer: refactoring
    // is a pain. Will fix later? Maybe with our data-generation
    // scheme this might be ideal.

    // TODO: auto-generate this in data.rs

    pub static ref TREE_POINTER: Mutex<Region> = Mutex::new(
        Region {
            reg_vec: None,
            coord_vec: vec![0.0; DIMS],
            half_length: 1.0,
            remove: false, // FIXME: remove?
            add_bucket: Some(vec![
                Body {
                    pos_vec: vec![-0.5, 0.0, 0.0],
                    vel_vec: vec![0.0, 0.0, 0.0],
                    mass: 1.0
                },

                Body {
                    pos_vec: vec![0.5, 0.0, 0.0],
                    vel_vec: vec![0.0, 0.0, 0.0],
                    mass: 1.0
                },
            ]),
            // add_bucket: None,
            com: None,
        }
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
