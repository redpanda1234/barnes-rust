// use std::thread;       // For fearless concurrency

// use std::fmt;

use super::data::*;
use super::physics::*;

use std::thread;
use std::sync::{Mutex, Arc};

#[derive(Clone, Debug)]
pub struct Body {
    pub pos_vec: Vec<f64>,
    pub vel_vec: Vec<f64>,
    pub mass: f64
}

#[derive(Clone, Debug)]
pub struct Region {
    pub reg_vec: Option<Vec<Arc<Mutex<Region>>>>,
    pub coord_vec: Vec<f64>,
    pub half_length: f64,
    pub add_queue: Option<Vec<Arc<Mutex<Body>>>>,
    pub com: Option<Arc<Mutex<Body>>>
}


impl Region {

    pub fn contains(&self, body_arc: Arc<Mutex<Body>>) -> bool {
        let body = &body_arc.lock().unwrap();

        for (qi, pi) in self.coord_vec.iter().zip(&body.pos_vec) {
            if (qi-pi).abs() > self.half_length {
                return false
            }
        }
        true
    }

    pub fn update(&mut self) {

    }

    fn split(&mut self) {


        match self.reg_vec {

            None => {

                let mut reg_vec = Vec::new();
                let quarter_length = self.half_length * 0.5;

                for vec in MULTIPLIERS.lock().unwrap().clone().iter_mut() {

                    for i in 0..DIMS {
                        vec[i] *= quarter_length;
                        vec[i] += self.coord_vec[i];
                    }

                    reg_vec.push(
                        Arc::new(
                            Mutex::new(
                                Region {
                                    reg_vec: None,
                                    // extract vec from mutable ref
                                    coord_vec: vec.to_vec(),
                                    add_queue: None,
                                    com: None,
                                    half_length: quarter_length,
                                }
                            )
                        )
                    )
                }
                self.reg_vec = Some(reg_vec);
            },
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
                 if self.half_length <= MIN_LEN {
                    let mut pos = vec![0.0; DIMS as usize];
                    let mut vel = vec![0.0; DIMS as usize];
                    let mut den = 0.0;

                    for mass in self.add_queue.clone().unwrap() {
                        // drop(match_me);
                        let mut com = mass.try_lock().unwrap();
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
                    self.com = Some(Arc::new(Mutex::new(Body {
                        pos_vec: pos,
                        vel_vec: vel,
                        mass: den
                    })));
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
                // never triggered because handeled in recurse
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
