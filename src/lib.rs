//! # What is This?
//!
//! Crayon is a small, portable and extensible game framework, which written with a minimalistic
//! modular design philosophy. Its built from the ground up to focus on cache friendly data
//! layouts in multicore environments with entity-component based architecture.
//!
//! It is loosely inspired by some amazing some amazing blogs on [bitsquid](https://bitsquid.blogspot.de),
//! [molecular](https://blog.molecular-matters.com) and [floooh](http://floooh.github.io/).
//!
//! Some goals include:
//!
//! - Extensible through external code modules;
//! - Run on macOS, Linux, Windows, iOS, Android from the same source;
//! - Stateless, layered, multithread render system with OpenGL(ES) 3.0 backends;
//! - Entity component system with a data-driven designs;
//! - Unified interface for handling input devices across platforms;
//! - Asynchronous data loading from various filesystem;
//! - etc.
//!
//! Please read the documents under modules for specific usages.
//!
//! ## Quick Example
//!
//! For the sake of brevity, you can also run a simple and quick example with commands:
//!
//! ```sh
//! git clone git@github.com:shawnscode/crayon.git && cd crayon/crayon-examples
//! cargo run imgui
//! ```

extern crate crossbeam_deque;
#[macro_use]
extern crate cgmath;
#[macro_use]
extern crate failure;
extern crate gl;
extern crate glutin;

#[macro_use]
extern crate serde;
extern crate bincode;
extern crate uuid;

#[macro_use]
pub mod errors;
#[macro_use]
pub mod utils;
pub mod application;
pub mod ecs;
#[macro_use]
pub mod video;
pub mod input;
pub mod math;
pub mod prelude;
pub mod res;
pub mod sched;

pub use errors::*;
