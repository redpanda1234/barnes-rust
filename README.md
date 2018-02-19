# Theo Mech Computational Project
A Rust-implemented Barnes-Hut n-body simulator. Project based on a
similar thing I made in Python last summer
(see [here](https://github.com/redpanda1234/euler)). Simulation is
written to be general over an arbitrary number of spatial dimensions;
working in 2D vs. 3D vs. nD is as simple as changing one global
constant (although graphics will remain 2D). All initial conditions
are generated at runtime, sampling scalar parameters such as mass,
speed, and distance-from-center from a choice of uniform, normal, or
gamma distributions. Speed and distance are then converted into
velocity and displacement by projecting down using nD spherical
coordinates. All major data structures are stored in mutexes, then
wrapped in thread-safe reference-counting pointers to allow
multi-threaded tree recursion in the future. Graphics implemented
using piston, a modular game engine library for Rust.

# How to run
Keep in mind, this is very much a work in progress. School is very
busy currently, so development will likely be paused until Harry and I
finish applying to REUs and such. And do keep in mind that we will
likely be doing most of our work on `develop`.

Anyways, if you do want to run this, you'll first want to have a
working rust install. Then, use git to clone the project by
```bash
git clone https://github.com/redpanda1234/barnes-rust.git
```
To run the project, cd into the `barnes-rust` directory, then do
```bash
cargo run
```
This should build the project, and execute the binary. If you really
want the simulation to run fast, and you're willing to wait for a few
extra seconds for the code to compile (on my laptop ~30 sec the first
time), you can run with
```bash
cargo run --release
```
Which will apply some quite significant optimizations that'll increase
performance and all that.


# Contributing
If you know anything about how to outsmart the borrow checker and/or
implement multithreading on tree structures, please open a pull
request.

# Some videos

+ [scattering off a binary star system](https://youtu.be/BcRRBVifNFI)
+ [an interesting bug we were wrestling with.](https://youtu.be/A_mKX2Y0R-c)
