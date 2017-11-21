// use std::thread;       // For fearless concurrency

// Static -> valid globally throughout the lifetime of the program
// mut allows us to modify the value contained in the static.
// TODO: implement a more intelligent thread limit thing.
static mut NUM_THREADS: i64 = 20;

// derive(Clone) tells rust to try and implement the clone trait on
// our Coord automatically. This allows us to clone the data inside of
// Coord later on in our program, without writing the method
// ourselves.

// TODO: implement a method for element-wise addition on Body

// TODO: implement some form of thread queue flagging at the end of
// each velocity update run.

// Body is going to end up being our class to represent masses. Each
// one will have a float vector to describe position, then some mass
// value assigned to it.
#[derive(Clone)]
pub struct Body {
    pub pos_vec: Vec<f64>,
    pub vel_vec: Vec<f64>,
    pub mass: f64,
}

/*
// MULTIPLIERS is a static array that we'll use later to quickly
// determine the centers of subregions when we recurse. If we multiply
// each of the sub-arrays in MULTIPLIERS by the sidelength of our
// region, then _add_ those to our position vector for our starting
// region, it'll get us the center of our new region. We'd like for
// there to be a more intelligent way of doing this, but if you
// examine the commit history, you'll see that we couldn't get it to
// work with auto-generated arrays.
*/
static MULTIPLIERS: [[f64; 2]; 4] = [
    [-1.0, -1.0],
    [-1.0, 1.0],
    [1.0, -1.0],
    [1.0, 1.0]
];

/*
// This is our top-level class that we'll use to represent regions in
// our recursive tree. We made the mistake of defining everything with
// option enums --- FIXME: refactor to make this not be the case.
// reg_vec is an optional vector of child regions --- if we're at a
// leaf in the tree, then this will be None, else we'll have a Some
// containing a vector of references to child regions.

// coord_vec is going to be a vector of floats describing the position
// of the center of our region (which is an n-dimensional box).

// half_length, as the name indicates, is going to be a float whose
// value is half of the length of a side of our box. We chose to use
// half lengths because it makes determining whether a region contains
// some mass faster.

// remove is a bool flag that will tell the update function whether or
// not the center-of-mass from the previous timestep in our tree is
// invalid or not. For instance, if any mass moves in the subtree, the
// COM is no longer valid. FIXME: I'm questioning whether this flag
// should even be a part of our struct or not. It seems that the com
// should _always_ change after a timestep.

// add_bucket is an optional queue for pushing masses into the region.
// The way our code currently works, when a mass enters some region,
// we push it into an add-queue for the region. Then, our region
// determines whether or not it needs to split into sub-regions, and
// if so, it splits and sequentially pushes the masses in its
// add-queue into the sub-queues for its children. In the end, the
// only region that'll actually do any work incorporating the mass
// will be the lowest-level sub-region. We call this model "corporate
// delegation."

// Finally, com is an optional Bodyinate that contains a position and
// a mass (center of mass of our region).

// ******** TODO / TOFIX ********
// + calculate distance metric in parent node
// + store at most one mass in the
// + create better implementations for generic-dimensional spaces
// + implement dropping of dead branches
// + collisions
//   - really really close bodies merge, but add a bonding energy
//     term to maintain conservation of energy
// + make com no longer an option enum
// + reimplement contains method by constructing indices using our
//   binary string construction method on the global multiplier array.
 */
#[derive(Clone)]
pub struct Region {
    pub reg_vec: Option<Vec<Region>>,
    pub coord_vec: Vec<f64>,
    pub half_length: f64,
    pub remove: bool, // FIXME: remove?
    pub add_bucket: Option<Vec<Body>>,
    pub com: Option<Body>,
}


// Let's implement methods on REgion!
impl Region {

    // contains takes some body, and then compares each of the i
    // coordinates in its position vector to determine whether it's
    // contained in the calling region or not.

    fn contains(&self, point: &Body) -> bool {

        // Iterate through all pairs of the i components of our
        // position coordinate

        for (qi, pi) in self.coord_vec.iter().zip(&point.pos_vec) {

            // TODO: make sure nothing funny happens if it happens to
            // be directly on the boundary... I think this is handeled
            // because we'll pop a mass as soon as it passes for one
            // of the regions, but let's double-check.

            if (qi-pi).abs() > self.half_length {
                return false
            }
        }
        true // implicit "return true" if it doesn't fail any checks
    }

    fn update(&mut self) -> i32 {
        match self.reg_vec.clone() {
            None => {
                if self.remove {
                    self.com = None;
                    match self.add_bucket.clone() {
                        None => 0,
                        Some(ref bucket) => {
                            if bucket.len() == 1 {
                                self.com = Some(bucket[0].clone());
                                1
                            } else {
                                self.recurse(true)
                            }
                        },
                    }
                } else {
                    match self.add_bucket.clone() {
                        None => 1,
                        Some(ref mut bucket) => {
                            bucket.push(self.com.clone().unwrap());
                            self.recurse(true)
                        },
                    }
                }
            },

            Some(mut _reg_vec) => {
                self.com = None;
                match self.add_bucket.clone() {
                    None => 1,
                    Some(ref _bucket) => {
                        let result = self.recurse(false);
                        if result == 0 {
                            self.reg_vec = None
                        }
                        result
                    },
                }
            },
        }
    }

    fn split(&mut self) {
        // TODO: parallelize stuff
        // if MULTIPLIERS.dumbass[0].len() != self.coord_vec.len() {
        //     panic!("Not enough frosh chem");
        // }

        let mut reg_vec = Vec::new();
        let quarter_length = self.half_length * 0.5;
        // let mult = self.populate_mult(2, 0.0);

        for vec in MULTIPLIERS.iter() {
            // have to define copy_pos this jenky way because we
            // defined our MULTIPLIERS as a static array
            let mut copy_pos = vec![vec[0], vec[1]];
            for i in 0..copy_pos.len() {
                copy_pos[i] += 0.5 * vec[i] * self.half_length;
            }
            reg_vec.push(
                Region {
                    reg_vec: None,
                    coord_vec: copy_pos,
                    remove: false,
                    add_bucket: None,
                    com: None,
                    half_length: quarter_length,
                }
            )
        }
        self.reg_vec = Some(reg_vec);
    }

    fn recurse(&mut self, split: bool) -> i32 {
        if split {self.split();}
        else {
            'outer: for mass in self.add_bucket.clone().unwrap() {
                'inner: for region in self.reg_vec.clone().unwrap() {
                    if region.contains(&mass) {
                        region.add_bucket.map(|mut v| v.push(mass));
                        break 'inner;
                    }
                }
            }
        }
        // self.add_bucket = None;

        let mut remove = 0;
        for mut region in self.reg_vec.clone().unwrap() {
            remove += region.update();
        }
        return remove;
    }
}
