extern crate rand;

use super::*;

// TODO: use this everywhere we check dimensions
pub const DIMS: usize = 3;
pub static THETA: f64 = 0.5;
pub static DT: f64 = 0.01;
pub static mut NUM_THREADS: i64 = 20;

static SEED: &[i8] = &[1, 2, 3, 4];

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

    static mut RNG: StdRng = SeedableRng::from_seed(SEED);

    // When testing, we'll want to run the same simulations
    // repeatedly, to make sure we're actually modifying how the
    // system behaves. get_seed is used to ensure that we'll be
    // seeding all random elements of the simulation with the same
    // pseudorandom conditions each time.

    // maybe this isn't necessary, but I wanted each of the random
    // generators to be using different rng iterators from each other?

    pub fn get_seed() -> f64 {
        RNG.gen::<f64>()
    }

    // Using a pseudo-randomly-generated scalar r, this function uses
    // n-d spherical coordinates to transform back into a r_vec in the
    // standard basis.

    // Here, T is a generic type representing arbitrary rng generators
    // for r and theta. rng is the seeded rng generator we'll be using
    // to get the values for stuff. The additional T1 is because the
    // last angle needs to be generated from a different range, and we
    // don't want to write something that matches on types of T.

    // T should range from 0 to pi, T1 from 0 to 2pi.

    // See HERE
    //
    // https://en.wikipedia.org/wiki/N-sphere#Spherical_coordinates
    //
    // for an explanation on what this function is _supposed_ to be
    // doing


    pub fn nd_spherical_r<T, T1>(
        r: f64,
        t_generator: T,
        t1_generator: T1,
        rng: StdRng
    ) -> Vec<f64> {

        let mut r_vec = vec![0.0, DIMS];

        // The final case is special, so we don't iterate all the way
        // through DIMS.

        // This'll hold the running product of sin values of each of
        // the thetas defining our position

        let mut product: f64 = 1.0;

        for i in 0..(DIMS-2) {

            let theta = T.ind_sample(&mut rng);
            r_vec[i] = r*(theta.cos())*product;

            // all future calculations will involve product of
            // preceding theta.sin() values, so we increment it here

            product *= theta.sin();

        }

        // The final theta value is special, as it ranges from 0 to
        // 2pi. So we treat it (and the corresponding r coordinates
        // whose definitions involve it) in special cases outside of
        // our loop.

        let theta = T1.ind_sample(&mut rng);
        r_vec[DIMS-2] = r * theta.cos() * product;

        // the final r_vec entry involves just .sin()'s, no .cos()'s.
        r_vec[DIMS-1] = r * theta.sin() * product;

        // return r_vec
        r_vec
    }

    // a generic body generator that takes generic random generators
    // of types R, M, and V.
    pub fn gen_body<R, V, M, T>(
        r_generator: R,
        v_generator: V,
        m_generator: M,
        t_generator: T
    ) -> Body {
        let mut rng: StdRng = SeedableRng::from_seed(get_seed());

        Body {
            pos_vec: nd_spherical_r(r_generator, t_generator, rng)
                vel_vec:
            mass:
        }
    }
    // pub mod r_funcs {

    //     pub fn linear_gen_r(rand_gen: Range) {
    //         // let range = rand::Range::new(10, 10000);
    //         // let mut rng = rand::thread_rng();
    //         // let mut sum = 0;
    //         // for _ in 0..1000 {
    //         //     sum += between.ind_sample(&mut rng);
    //         // }
    //         // println!("{}", sum);
    //     }

    //     fn gamma_gen_r() {

    //     }

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
        gen_populate_mult(DIMS, 0.0)
    );
}
