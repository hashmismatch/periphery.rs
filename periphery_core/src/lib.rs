//! The basic framework for connecting to external peripheries, with an infrastructure
//! to easily detect, inspect and interact with devices and sensors. Fully abstracts
//! the system's environment.

#![allow(warnings)]

#![cfg_attr(not(feature = "std"), no_std)]

#![cfg_attr(not(feature="std"), feature(alloc))]
#![cfg_attr(not(feature="std"), feature(collections))]
#![cfg_attr(not(feature="std"), feature(core_intrinsics))]
#![cfg_attr(not(feature="std"), feature(slice_concat_ext))]

#[cfg(not(feature="std"))]
mod float_core; 

#[cfg(not(feature="std"))]
#[macro_use]
extern crate alloc;

#[cfg(not(feature="std"))]
#[macro_use]
extern crate collections;


extern crate periphery_buspirate_parser;


pub extern crate terminal_cli;


extern crate packed_struct;

//#[macro_use]
//extern crate packed_struct_codegen;

pub mod prelude;

mod base;
pub mod bus;
pub mod cli;
pub mod system;


#[macro_use]
mod register;

//#[macro_use]
//mod bus_registers;

pub mod device;
pub mod device_factory;
pub mod device_storage;
pub mod units;

// pub mod buspirate;

pub mod utils;

pub use register::*;
pub use base::*;

