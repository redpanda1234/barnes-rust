use std::thread;       // For fearless concurrency

// Static -> valid globally throughout the lifetime of the program
// mut allows us to modify the value contained in the static
static mut NUM_THREADS: i64 = 20; // TODO: intelligent thread limit

pub struct Coord {
    x: f64,
    y: f64,
    mass: f64,
}

#[derive(Debug)] // for printing stuff
pub struct Region {
    pub reg_vec: Option<Vec<Region>>,

    pub x: f64, // use the top-right corner as the reference point
    pub y: f64, //
    pub length: f64,

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

    fn contains(&self, point: &Point) -> bool {

        // Since rust is an expression language thing, then the
        // last evaluated exp (below, a bool) will be the return
        // value of the function
        point.x <= self.x + self.length && point.x >= self.x &&
            point.y <= self.y + self.length && point.y >= self.y
    }

    fn split(&mut self) {       // TODO: parallelize stuff
        let side: f64 = self.length/2;
        self.reg_vec = Some(vec![
            // ne
            Region {
                reg_vec: None,

                x: self.x,
                y: self.y,
                length: side,

                remove: false,
                add_bucket: None,

                com: None,
            },

            // nw
            Region {
                reg_vec: None,

                x: self.x - side,
                y: self.y,
                length: side,

                remove: false,
                add_bucket: None,

                com: None,
            },

            // sw
            Region {
                reg_vec: None,

                x: self.x - side,
                y: self.y - side,
                length: side,

                remove: false,
                add_bucket: None,

                com: None,
            },

            // se
            Region {
                reg_vec: None,

                x: self.x,
                y: self.y - side,
                length: side,

                remove: false,
                add_bucket: None,

                com: None,
            }
        ])
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
                if remove {
                    if self.add_bucket.len() == 1 {
                        // If we're removing it anyways, just redefine
                        self.com = self.add_bucket[0];
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
                    match self.addbucket {
                        None => None,
                        Some(bucket) => self.split(),
                    }
                }
            },

            // Done with the None case, now we move to the some case

            Some(reg_vec) => {

            },
        }
    }
}
