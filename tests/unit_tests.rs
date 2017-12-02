// // extern crate physics;
// // extern crate tree;
// // extern crate data;
// use super::physics::*;
// use super::tree::*;

// #[cfg(test)]
// mod physics_tests {

//     //
//     #[test]
//     fn test_dist_sq() {
//         let m1 = Body {
//             pos_vec: vec![1.0, 0.0, 0.0],
//             vel_vec: vec![0.0, 0.0, 0.0],
//             mass: 0.0
//         };

//         let m2 = Body {
//             pos_vec: vec![0.0, 0.0, 0.0],
//             vel_vec: vec![0.0, 0.0, 0.0],
//             mass: 0.0
//         };

//         let m3 = Body {

//             pos_vec: vec![-3.0, 0.0, 0.0],
//             vel_vec: vec![0.0, 0.0, 0.0],
//             mass: 0.0
//         };

//         let m4 = Body {
//             pos_vec: vec![0.0, 4.0, 0.0],
//             vel_vec: vec![0.0, 0.0, 0.0],
//             mass: 0.0
//         };

//         assert_eq!(m1.squared_dist_to(&m2), 1.0);
//         assert_eq!(m3.squared_dist_to(&m4), 25.0);
//     }

//     #[test]
//     fn test_vec_rel() {
//         let m1 = Body {
//             pos_vec: vec![1.0; DIMS],
//             vel_vec: vec![0.0; DIMS],
//             mass: 0.0
//         };

//         let m2 = Body {
//             pos_vec: vec![0.0; DIMS],
//             vel_vec: vec![0.0; DIMS],
//             mass: 0.0
//         };

//         // let m3 = Body {
//         //     pos_vec: vec![-3.0; DIMS],
//         //     vel_vec: vec![0.0; DIMS],
//         //     mass: 0.0
//         // };

//         // let m4 = Body {
//         //     pos_vec: vec![4.0].extend([0.0; DIMS-1].iter()),
//         //     vel_vec: vec![0.0; DIMS],
//         //     mass: 0.0
//         // };
//         println!("m1 rel m2 {:?}", m1.vec_rel(&m2));

//         assert_eq!(m1.vec_rel(&m2), vec![-1.0; DIMS]);
//         // assert_eq!(m3.vec_rel(&m4), vec![7.0].extend(vec![0.0; DIMS-1]));
//     }

//     #[test]
//     fn test_sq_mag() {
//         let m1 = Body {
//             pos_vec: vec![1.0, 0.0, 0.0],
//             vel_vec: vec![0.0, 0.0, 0.0],
//             mass: 0.0
//         };

//         let m2 = Body {
//             pos_vec: vec![0.0, 0.0, 0.0],
//             vel_vec: vec![0.0, 0.0, 0.0],
//             mass: 0.0
//         };

//         let m3 = Body {
//             pos_vec: vec![-3.0, 0.0, 0.0],
//             vel_vec: vec![0.0, 0.0, 0.0],
//             mass: 0.0
//         };

//         let m4 = Body {
//             pos_vec: vec![0.0, 4.0, 0.0],
//             vel_vec: vec![0.0, 0.0, 0.0],
//             mass: 0.0
//         };
//         // println!("m1 rel m2 {:?}", m1.vec_rel(&m2));

//         assert_eq!(m1.sq_magnitude(&m1.vec_rel(&m2)), 1.0);
//         assert_eq!(m3.sq_magnitude(&m3.vec_rel(&m4)), 25.0);
//     }
// }
