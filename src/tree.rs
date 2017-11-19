// use std::thread;       // For fearless concurrency

// Static -> valid globally throughout the lifetime of the program
// mut allows us to modify the value contained in the static
static mut NUM_THREADS: i64 = 20; // TODO: intelligent thread limit

#[derive(Clone)]
pub struct Coord {
    pub pos_vec: Vec<f64>,
    mass: f64,

    // TODO: implement a method for element-wise addition on coord
}

// fn populate_mult(n: i8, mult: f64) -> Vec<Vec<f64>> {
//     if n <= 0 {
//         return vec![vec![mult]];
//     }

//     let mut v1: Vec<Vec<f64>> = populate_mult(n - 1, -1.0);
//     v1.extend(populate_mult(n - 1, 1.0));

//     if mult != 0.0 {
//         for i in 0..v1.len() {
//             v1[i].push(mult);
//         }
//     }

//     v1
// }

// TODO: create a value for this in main

// static MULTIPLIERS: Vec<Vec<f64>> = vec![vec![]];
// pub struct Bullshit {
//     dumbass: Vec<Vec<f64>>,
// }

// static MULTIPLIERS: Bullshit = Bullshit {dumbass: populate_mult(2, 0.0)};
  // ((-1,-1), (-1,1), (1,-1), (1,1));
  // ((f64), (f64), (f64), (f64))

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

    fn populate_mult(&mut self, n: i8, mult: f64) -> Vec<Vec<f64>> {
        if n <= 0 {
            return vec![vec![mult]];
        }

        let mut v1: Vec<Vec<f64>> = self.populate_mult(n - 1, -1.0);
        v1.extend(self.populate_mult(n - 1, 1.0));

        if mult != 0.0 {
            for i in 0..v1.len() {
                v1[i].push(mult);
            }
        }

        v1
    }


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
        let pos_vec = point.pos_vec.clone();
        for (qi, pi) in self.coord_vec.iter().zip(pos_vec) {
            if (qi-pi).abs() > self.half_length {
                return false
            }
        }
        true
    }

    fn split(&mut self) { // TODO: parallelize stuff
        // if MULTIPLIERS.dumbass[0].len() != self.coord_vec.len() {
        //     panic!("Not enough frosh chem");
        // }

        let mut reg_vec = Vec::new();
        let quarter_length = self.half_length * 0.5;
        let mult = self.populate_mult(2, 0.0);

        for vec in mult.iter() {
            let mut copy_pos = vec.clone();
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

    fn update(&mut self) -> bool {
        match self.reg_vec {
            // TODO: if remove is true for all children,
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
                    self.com = None;
                    match self.add_bucket {
                        // If our node is totally empty, prune it
                        None => true,
                        // else ingest the queued masses
                        Some(ref mut bucket) => {
                            if bucket.len() == 1 {
                                self.com = Some(bucket[0].clone());
                                false
                            } else {
                                self.split();
                                self.recurse()
                            }
                        },
                    }
                } else {
                    match self.add_bucket {
                        None => false,
                        Some(ref mut bucket) => {
                            bucket.push(self.com.clone().unwrap());
                            self.split();
                            self.recurse()
                        },
                    }
                }
            },

            // Done with the None case, now we move to the some case, @Raxod502

            Some(ref reg_vec) => {
                self.com = None;
                match self.add_bucket {
                    None => false,
                    Some(ref bucket) => {self.recurse()},
                }
            },
        }
    }

    fn recurse(&mut self) -> bool {
        'outer: for mass in self.add_bucket.unwrap() {
            'inner: for region in self.reg_vec.unwrap() {
                if region.contains(&mass) {
                    region.add_bucket.map(|mut v| v.push(mass));
                    break 'inner;
                }
            }
        }
        self.add_bucket = None;

        let mut remove = false;
        for region in self.reg_vec.unwrap() {
            if region.update() { remove = true; }
        }
        return remove;
    }

}
