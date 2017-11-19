use std::thread;       // For fearless concurrency

// Static -> valid globally throughout the lifetime of the program
// mut allows us to modify the value contained in the static
static mut NUM_THREADS: i64 = 20; // TODO: intelligent thread limit

pub struct Coord {
    pub pos_vec: Vec<f64>,
    mass: f64,

    // TODO: implement a method for element-wise addition on coord
}

static MULTIPLIERS: Vec<i8> = Vec::new(); // multipliers for
// constructing the subregion vectors

// Note: radon suggests this is dumb. Instead, maybe construct by
// starting with an initial vector
fn populate_mult() -> Vec<Vec<i8>> {
    let num_dim = 2;
    let initial = Vec::new();

    for n in [0..num_dim] {
        initial.extend(1)
    }

    let output = vec![&initial];

    'outer: loop {
        'inner: for i in [0..num_dim] {
            if (output.last()[i] == 1) {
                output.push(&vec.clone_from(initial));
            } else {
                ouptut.push(&)
            }
        }
    }
}


pub struct Region {
    pub reg_vec: Option<Vec<Region>>,

    // The vector of coordinates defining the position of the minimum
    // corners of our n-box
    pub coord_vec: Vec<f64>,
    pub half_length: f64,

    pub remove: bool,
    pub add_bucket: Option<Vec<Coord>>, // masses to inject

    pub com: Option<Coord>,
}


impl Region {

    // TODO: calculate distance metric in parent node
    // store at most one mass in the

    // possible TODO: implement in 4-D and project down

    // contains takes a reference to the self struct and a point
    // struct, then determines whether point is contained within
    // the bounds of region.

    // TODO: implement dropping of dead branches

    // TODO: collisions

    // Also: really really close bodies merge, but add the bonding
    // energy term

    fn contains(&self, point: &Coord) -> bool {
        for (qi, pi) in self.coord_vec.zip(point.pos_vec) {
            if (qi-pi).abs() > self.half_length {
                return false
            }
        }
        true
    }

    fn split(&mut self) { // TODO: parallelize stuff
        let self.reg_vec = Some(vec![

        ]);
    }

    fn update(&mut self) {
        match self.reg_vec {

            // Some very labyrinthine control flow here. Hopefully
            // it's well-documented at the very least.

            // If the region vector is None, then we have no current
            // children subtree, and we need to decide how best to
            // update it. There are a few options.

            // 1. The mass that formerly occupied this box has moved
            // out of it. If so, we then need to decide whether to
            //
            // (a) prune this node
            // (b) only modify this node (and no subtrees)
            // (c) draw in subtrees for this node

            // These cases are handled by the pattern block below.

            // TODO: refactor this dumbass method by making a separate
            // method to handle the addlist (verbosity sucks)

            None => {
                // If the mass has been flagged for removal
                if self.remove {
                    let add_bucket = self.add_bucket.unwrap();
                    if add_bucket.len() == 1 {
                        // If we're removing it anyways, just redefine
                        self.com = Some(add_bucket[0]);
                    } else {
                        // Empty the COM; we'll redefine it if need
                        self.com = None;

                        match self.add_bucket {
                            // If our node is totally empty, prune it
                            None => self.prune(),
                            // else ingest the queued masses
                            Some(bucket) => self.split(),
                        }
                    }
                } else {
                    match self.add_bucket {
                        None => (),
                        Some(bucket) => self.split(),
                    }
                }
            },

            // Done with the None case, now we move to the some case

            Some(reg_vec) => {
                match self.add_bucket {
                    None => (),
                    Some(bucket) => self.recurse(),
                }
            },
        }
    }
}
