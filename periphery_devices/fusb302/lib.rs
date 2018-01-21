#[macro_use]
extern crate periphery_core;

extern crate packed_struct;

#[macro_use]
extern crate packed_struct_codegen;

extern crate usb_pd;

pub mod registers;

mod device;
pub use self::device::*;
