#![feature(custom_attribute)]

#[macro_use]
extern crate periphery_core;

extern crate packed_struct;

#[macro_use]
extern crate packed_struct_codegen;

extern crate gesture_detection;

extern crate autogain;

mod registers;
mod sensor;
mod integration;

pub use self::registers::*;
pub use self::sensor::*;
pub use self::integration::*;
