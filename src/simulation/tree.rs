use std::thread;       // For fearless concurrency

// Static -> valid globally throughout the lifetime of the program
// mut allows us to modify the value contained in the static
static mut NUM_THREADS: i64 = 20; // TODO: intelligent thread limit

#[derive(Debug)] // for printing stuff
pub struct Region {
    pub reg_vec: Option<Vec<Option<Region>>>,

    /* Dead code?
    // pub ne: Option<region>, // Option enum; either some or none.
    // pub nw: Option<region>, // Makes handling dead regions easier.
    // pub sw: Option<region>,
    // pub se: Option<region>,
    */

    pub x: f64, // use the bottom-left corner as the reference point
    pub y: f64, //
    pub length: f64,

    pub remove: bool,
    pub add_bucket: Option<Vec<Coord>>, // masses to inject

    pub com: Option<Coord>,
}

pub struct Coord {
    x: f64,
    y: f64,
    mass: f64,
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

    fn update(&self) -> Coord {
        match self.reg_vec {
            None => {
                if
            }

            Some() => {

            }
        }
    }
}
