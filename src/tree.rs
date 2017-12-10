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
    pub mass: f64
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
// + create better implementations for generic-dimensional spaces
// + implement dropping of dead branches
// + collisions
//   - really really close bodies merge, but add a bonding energy
//     term to maintain conservation of energy
// + reimplement contains method by constructing indices using our
//   binary string construction method on the global multiplier array.
 */
use std::sync::{Mutex, Arc};
use std::thread;

// use std::rc::Rc;
// use std::cell::RefCell;

#[derive(Clone, Debug)]
pub struct Region {
    pub reg_vec: Option<Vec<Arc<Mutex<Region>>>>,
    pub coord_vec: Vec<f64>,
    pub half_length: f64,
    pub add_queue: Option<Vec<Arc<Mutex<Body>>>>,
    pub com: Option<Arc<Mutex<Body>>>
}


// Let's implement methods on REgion!
impl Region {

    // contains takes some body, and then compares each of the i
    // coordinates in its position vector to determine whether it's
    // contained in the calling region or not.


    pub fn contains(&self, body_arc: Arc<Mutex<Body>>) -> bool {
        //println!("called contains");
        let body = &body_arc.lock().unwrap();
        // Iterate through all pairs of the i components of our
        // position coordinate

        for (qi, pi) in self.coord_vec.iter().zip(&body.pos_vec) {

            // TODO: make sure nothing funny happens if it happens to
            // be directly on the boundary... I think this is handeled
            // because we'll pop a mass as soon as it passes for one
            // of the regions, but let's double-check.

            if (qi-pi).abs() > self.half_length {
                // println!("shit, return false");
                return false
            }
        }
        true // implicit "return true" if it doesn't fail any checks
    }

    // update does two jobs at once. It recursively pushes masses from
    // add queues
    pub fn update(&mut self) -> i32 {

        // First check whether the calling region has any child
        // regions. This will determine how we handle our updating.
        // Currently, we check whether

        match self.reg_vec.clone() {

            None => {

                // if we don't have a defined region vector, then that
                // means that either we're a leaf node, or we're doing
                // the first initial push of masses into the tree.

                // we want to check whether there are any new
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

                    None => {
                        // println!("nothing to add");
                        match &self.com {
                            &None => 0,
                            &Some(_) => 1
                        }
                    },

                    // if our add_queue is nonempty, then we need
                    // to handle ingesting of the masses.

                    Some(mut queue) => {

                        match self.com.clone() {

                            None => self.recurse(true),

                            // If we have a current com, we push
                            // it into the queue (because we're
                            // still at a leaf node), and then
                            // subdivide accordingly, returning
                            // the number of submasses contained.

                            Some(mut com) => {
                                queue.push(com);
                                self.com = None;
                                self.add_queue = Some(queue);
                                self.recurse(true)
                            },
                        }
                    },
                }
            },

            // Case that our region has a defined vector of child
            // regions. TODO: check for dead regions, and prune those.
            // Perhaps we should make each of the entries in the
            // vector options on Regions?

            Some(mut reg_vec) => {

                // Invalidate com, because it's gonna be invalid no
                // matter what if we aren't at a leaf node.

                self.com = None;

                //
                match &self.add_queue {

                    // If the add_queue is None, we only want to look
                    // at the child regions.

                    &None => {
                        //println!("updating children");
                        let mut return_me = 0;
                        for reg_arc in reg_vec.iter() {
                            let mut reg = reg_arc.lock().unwrap();
                            return_me += reg.update();
                        }
                        if return_me == 0 {
                            // println!("\n\nDeleted region vector: {:#?}\n\n", self.coord_vec);
                            self.reg_vec = None;
                        }
                        self.reg_vec = Some(reg_vec);
                        return_me
                    },

                    // I think this case should never be called
                    // because the way we inject masses should mean they
                    // always go into leaf nodes
                    // right????
                    &Some(_) => {
                        // println!("whee!");
                        // for some reason, this case is never
                        // reached. (or is it?)
                        // println!("injecting bodies into child regions");
                        // recurse on false because we don't need to
                        // split the region (it's already splitted)
                        let result = self.recurse(false);
                        if result == 0 {
                            // println!("\n\nDeleted region vector: {:#?}\n\n", self.coord_vec);
                            // self.reg_vec = None
                        }
                        result
                    },
                }
            },
        }
    }

    fn split(&mut self) {
        // println!("\n\n\n\nsplitting self: \n {:#?}", self);
        // First, we check whether the calling region has any child
        // regions or not. Don't want to split a region that's already
        // been split.

        match self.reg_vec {

            None => {

                // There's no children (as expected), so we'll create
                // the children vector. First, we need to make a
                // placeholder vector to write to, since None can't be
                // unwrapped.

                let mut reg_vec = Vec::new();

                // quarter_length will be used to efficiently
                // calculate the coordinate vectors of the
                // newly-created child regions. We compute
                // it up here so that we don't have to during each
                // iteration of the loop below.

                let quarter_length = self.half_length * 0.5;

                // MULTIPLIERS is stored in a Mutex, so we have to
                // lock and unwrap it, then clone it so that we can
                // use its data while freeing it for other waiting
                // threads to access threads can access it once we're
                // done. FIXME: why do we need a mutex for multipliers
                // at all...? I can't see why we'd be modifying it.
                // I think it'd be faster to just pass a reference to
                // the global MULTIPLIERS vector everywhere than it is
                // locking and unwrapping it.

                for vec in MULTIPLIERS.lock().unwrap().clone().iter_mut() {

                    // Here, MULTIPLIERS represents all the possible
                    // displacement vectors between the center of the
                    // calling region and the centers of its child
                    // regions, scaled by a factor of quarter_length.

                    for i in 0..DIMS {
                        vec[i] *= quarter_length;
                        vec[i] += self.coord_vec[i];
                    }

                    // Construct the empty child region corresponding
                    // to the relative position given by vec, and push
                    // it into the calling region's region vector

                    reg_vec.push(
                        Arc::new(
                            Mutex::new(
                                Region {
                                    reg_vec: None,
                                    // vec is currently a mutable reference,
                                    // so we call .to_vec() on it to extrac
                                    // the underlying vec.
                                    coord_vec: vec.to_vec(),
                                    add_queue: None,
                                    com: None,
                                    half_length: quarter_length,
                                }
                            )
                        )
                    )
                }

                // Now that we're done pushing all the child regions,
                // we write reg_vec out to the calling region, and
                // place it into an Option enum so that we can perform
                // matches on it later.

                self.reg_vec = Some(reg_vec);

            },

            // We shouldn't ever be calling split on a pre-split
            // region, so one of the calls must've been wrong.
            Some(_) => {
                panic!("this wasn't supposed to happen. {:#?}", self);
            }
        }
    }

    fn recurse(&mut self, split: bool) -> i32 {
        // println!("\n\n\nrecursing with {} on self: \n{:#?}", split, self);
        // we call recurse(true) only when we need to split the
        // region, so first call split then recurse on false.
        //println!("called recurse");

        if split {


            if self.add_queue.clone().unwrap().len() == 1 {

                //note that we can overwrite com because it
                //would already have been added to
                //the add queue, if it existed

                self.com = self.add_queue.clone().unwrap().pop();
                self.add_queue = None;
                return 1

            } else {

                //if this region is very small and we don't want to subdivide it
                //further, combine all the masses here into one
                 if self.half_length < MIN_LEN {
                    let mut pos = vec![0.0; DIMS as usize];
                    let mut vel = vec![0.0; DIMS as usize];
                    let mut den = 0.0;

                    for mass in self.add_queue.iter() {
                        let mut match_me = mass.try_lock().unwrap().com;
                        // println!("{:#?}", match_me);
                        match &match_me {
                            &None => continue,
                            &Some(ref com_arc) => {
                                // drop(match_me);
                                let mut com = com_arc.try_lock().unwrap();
                                den += com.mass.clone();
                                //TODO: we shouldn't have to be cloning pos_vec
                                pos =pos
                                    .iter()
                                    .zip(com.pos_vec.clone())
                                    .map(|(pi, pv)| pi + pv * com.mass)
                                    .collect::<Vec<f64>>();
                                vel =vel
                                    .iter()
                                    .zip(com.vel_vec.clone())
                                    .map(|(pi, pv)| pi + pv * com.mass)
                                    .collect::<Vec<f64>>();
                            },
                        }
                    }
                    //if we didn't add any masses, make sure we're not dividing by 0
                    if den != 0.0 {
                        pos = pos
                            .iter()
                            .map(|n| n / den)
                            .collect::<Vec<f64>>();
                        vel = vel
                            .iter()
                            .map(|n| n / den)
                            .collect::<Vec<f64>>();
                    }
                    self.add_queue = None;
                    self.com = Arc::new(Mutex::new(Body {

                    }));
                    return 1;
                } else {
                    self.split();
                    return self.recurse(false)
                }
            }

        } else {
            self.push_masses_to_children();

            let mut remove = 0;

            match self.reg_vec.clone() {

                None => {
                    match self.com {
                        None => 0,
                        Some(_) => 1
                    }
                },

                Some(mut reg_vec) => {

                    for reg_arc in reg_vec.iter() {
                        let mut region = reg_arc.lock().unwrap();
                        // println!("updating child regions");
                        remove += region.update();
                    }
                    // println!("child regions are {:#?}", reg_vec);
                    self.reg_vec = Some(reg_vec);

                    //println!("returning {:#?} from recurse false", remove);
                    return remove;
                }
            }
        }
    }

    //remove masses from the add_queue
    //and place them in the child nodes
    //FIXME: not properly removing masses that aren't in the
    //simulation anymore
    pub fn push_masses_to_children(&mut self) {
        // println!("pushing masses to children");
        // FIXME: do this actually properly.

        // we clone the queue so that we can pop masses out of it
        // as we're working. Shouldn't need to do this though.

        let mut queue = self.add_queue.clone().unwrap();
        let queue_len = queue.clone().len();

        // take the raw Option enum on the vector of child regions
        // and not the wrapped vector itself, so that we can do
        // checks on it first to make sure it's nonempty.

        let opt_vec = self.reg_vec.clone();

        match opt_vec {

            None => {
                // If we don't have a set of child regions, then we
                // only want what's on the top of the queue (there
                // should only be one thing, else we'd have split in
                // the self.recurse() call)
                self.com = queue.pop();
                // never triggered because handeled in recurse. FIXME
                panic!("aaaa! why isn't this case ever triggered???");
                // println!("None case com is \n{:#?}", self.com);
                assert_eq!(queue.len(), 0);
            },

            Some(mut reg_vec) => {
                // let vec_len = reg_vec.len();

                'outer: for _ in 0..queue_len {
                    // println!("this mass is \n{:#?}", mass.clone());
                    // println!("the queue is {:#?}", queue.clone());
                    // println!("the current region vector is {:#?}", reg_vec);

                    let m_arc = queue.pop().unwrap();

                    'inner: for reg_arc in reg_vec.iter() {
                        let mut region = reg_arc.lock().unwrap();
                        if region.contains(Arc::clone(&m_arc)) {
                            // define reg_queue here for the Some arm
                            // of our match
                            let mut reg_queue = region.add_queue.clone();

                            let mut reg_queue = match reg_queue {
                                None => {Some(Vec::new())},
                                Some(_) => {reg_queue},
                            }.unwrap();

                            reg_queue.push(m_arc);
                            region.add_queue = Some(reg_queue);
                            continue 'outer
                        }
                    }
                }
                self.reg_vec = Some(reg_vec);
            }
        }
        self.add_queue = None;
    }

    // push a body to this region's add_queue
    pub fn push_body_global(body_arc: Arc<Mutex<Body>>) {
        let ref tree = &TREE_POINTER.lock().unwrap().tree.clone();
        let mut add_queue = tree.add_queue.clone();

        //if the added mass is outside of the tree region, don't add it
        // println!("about to call contains");
        if(!tree.contains(Arc::clone(&body_arc))) {
            // panic!("wwaaaa");
            // println!("\n\nDeleted mass: {:#?}\n\n", body_arc);
            return;
        } else {
            // panic!("panci");
            // println!("didn't delete mass\n\n\n\n\n\n\n\n\n");
            //if the add queue doesn't already exist, create it
            match add_queue {

                None => {
                    let mut queue = Vec::new();
                    queue.push(body_arc);
                    TREE_POINTER.lock().unwrap().tree.add_queue = Some(queue);
                },

                Some(mut queue) => {
                    queue.push(body_arc);
                    TREE_POINTER.lock().unwrap().tree.add_queue = Some(queue);
                }
            };
            //println!("TREE add queue: {:#?}", TREE_POINTER.lock().unwrap().tree.add_queue.clone());

        }
    }

    pub fn list_masses(&self) -> Vec<Body> {

        match self.reg_vec.clone() {
            None => {
                match self.com.clone() {
                    None => vec![],
                    Some(ref mut com) => vec![com.lock().unwrap().clone()]
                }
            },
            Some(ref reg_vec) => {
                let mut result = Vec::new();
                for mut child_arc in reg_vec {
                    let child = child_arc.lock().unwrap();
                    result.append(&mut child.list_masses());
                }
                result
            }
        }
    }
}
