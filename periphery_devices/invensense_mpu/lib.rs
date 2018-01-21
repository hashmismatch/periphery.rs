#[macro_use]
extern crate periphery_core;

extern crate packed_struct;

#[macro_use]
extern crate packed_struct_codegen;

mod sensor;
mod registers;

pub use self::sensor::*;
pub use self::registers::*;