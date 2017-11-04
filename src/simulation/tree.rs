extern crate num_cpus; // So that we can know how many processors to work with
use std::thread;       // For fearless concurrency


// Static -> valid globally throughout the lifetime of the program
// mut allows us to modify the value contained in the static
static mut NUM_THREADS: i64 = 20; // magic number for now, will find more
                                  // intelligent thread spawner later

#[derive(Debug)] // for printing stuff
pub struct Region {
    pub reg_vec: Option<Vec<Option<Region>>> // FIXME: make this syntactically valid

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
    fn contains(&self, point: &Point) -> bool {
        if point.x <= self.x + self.length && point.x >= self.x {
            // Since rust is an expression language thing, then the
            // last evaluated exp (below, a bool) will be the return
            // value of the function
            point.y <= self.y + self.length && point.y >= self.y
        }
        // else return false
        false
    }

    fn calc_com(&self) -> Coord {
        if remove {
            self.com = None
        }
    }
}
