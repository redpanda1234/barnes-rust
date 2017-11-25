// use std::thread;       // For fearless concurrency

// use std::fmt;

use super::data::*;
use super::physics::*;

// Static -> valid globally throughout the lifetime of the program
// mut allows us to modify the value contained in the static.
// TODO: implement a more intelligent thread limit thing.

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

#[derive(Clone, Debug)]
pub struct Body {
    pub pos_vec: Vec<f64>,
    pub vel_vec: Vec<f64>,
    pub mass: f64,
}

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

// add_queue is an optional queue for pushing masses into the region.
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

#[derive(Clone, Debug)]
pub struct Region {
    pub reg_vec: Option<Vec<Region>>,
    pub coord_vec: Vec<f64>,
    pub half_length: f64,
    pub remove: bool, // FIXME: remove?
    pub add_queue: Option<Vec<Body>>,
    pub com: Option<Body>
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

    // update does two jobs at once. It recursively pushes masses from
    // add queues
    pub fn update(&mut self) -> i32 {

        // println!("updating {:?}", self);
        // println!("helooooo");

        // First check whether the calling region has any child
        // regions. This will determine how we handle our updating.
        // Currently, we check whether

        match self.reg_vec.clone() {

            None => {

                // println!("rv: None {:?}", self);

                // if we don't have a defined region vector, then that
                // means that either we're a leaf node, or we're doing
                // the first initial push of masses into the tree.

                // The remove flag tells us whether or not the current
                // COM defined in our object is no longer valid. This
                // would happen if we need to redefine the center of
                // mass, e.g. if one of the sub-masses in the tree has
                // moved into a different region.

                if self.remove {

                    // println!("rv: None. rem: 1 {:?}", self);

                    self.com = None;

                    // Now we want to check whether there are any new
                    // masses waiting to be added to our region. If
                    // there aren't, we return 0 (because Harry had
                    // the idea of using our recursive update function
                    // to simultaneously calculate how many bodies
                    // were contained in subregions of our region, as
                    // idea of calculating a metric for the number of
                    // bodies contained below the given body, which
                    // will be useful in multithreading), else we
                    // recurse down into the tree.

                    match self.add_queue.clone() {

                        None => 0,

                        // if our add_queue is nonempty, then we need
                        // to handle ingesting of the masses.

                        Some(ref mut queue) => {

                            // println!("rv: None. rem: 1. aq: S {:?}", self);

                            // If we only have one mass in the queue,
                            // then we can just store it as the center
                            // of mass of our entire Region. Also,
                            // this means we don't need to recurse
                            // down at all.

                            if queue.len() == 1 {

                                self.com = Some(queue[0].clone());
                                self.add_queue = None; // clear queue
                                1 // There's one body stored below

                            } else {

                                // else, we want to recursively inject
                                // the masses
                                self.recurse(true)

                            }
                        },
                    }
                } else {

                    // println!("rv: None. rem: 0 {:?}", self);

                    // if we don't have to modify the current
                    // com...hang on, that can't be right. We'll
                    // always need to modify the com. FIXME! Unless
                    // handle updating the com's of leaf nodes
                    // directly. Whi

                    match self.add_queue.clone() {

                        // If the add queue is empty, we still need to
                        // update the single body that's in the
                        // calling region, which will just be
                        // self.com

                        None => {self.update_com(); 1},

                        // else, we need to recursively ingest the
                        // masses.

                        Some(ref mut queue) => {

                            match self.com.clone() {

                                // This doesn't make a great deal of
                                // sense. In fact, I think it makes no
                                // sense. We need to recurse down the
                                // tree if we have a com that doesn't
                                // need removing, and update the
                                // calling region's com accordingly.
                                // Here's a possibly bug-filled
                                // implementation.

                                None => {
                                    let return_me = self.recurse(true);
                                    self.update_com();
                                    return_me
                                },

                                // If we have a current com, we push
                                // it into the queue (because we're
                                // still at a leaf node), and then
                                // subdivide accordingly, returning
                                // the number of submasses contained.

                                Some(_com) => {
                                    queue.push(self.com.clone().unwrap());
                                    let return_me = self.recurse(true);
                                    self.update_com();
                                    return_me
                                }
                            }
                        },
                    }
                }
            },

            // Case that our region has a defined vector of child
            // regions. TODO: check for dead regions, and prune those.
            // Perhaps we should make each of the entries in the
            // vector options on Regions?

            Some(mut reg_vec) => {

                self.com = None;
                match self.add_queue.clone() {

                    None => {
                        let mut return_me = 0;

                        for reg in reg_vec.iter_mut() {
                            return_me += reg.update();
                        };

                        self.reg_vec = Some(reg_vec);
                        return_me
                    },

                    Some(ref _queue) => {
                        // recurse on false because we don't need to
                        // split the region (it's already splitted)
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

        for vec in MULTIPLIERS.lock().unwrap().clone().iter() {
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
                    add_queue: None,
                    com: None,
                    half_length: quarter_length,
                }
            )
        }
        self.reg_vec = Some(reg_vec);
    }

    fn recurse(&mut self, split: bool) -> i32 {
        if split {self.split(); self.update();}
        else {
            'outer: for mass in self.add_queue.clone().unwrap() {
                'inner: for region in self.reg_vec.clone().unwrap() {
                    if region.contains(&mass) {
                        region.add_queue.map(|mut v| v.push(mass));
                        break 'inner;
                    }
                }
            }
        }
        // self.add_queue = None;

        let mut remove = 0;
        for mut region in self.reg_vec.clone().unwrap() {
            remove += region.update();
        }
        return remove;
    }
}
