//! Traits for a particular device connected to a bus. Macros and abstractions to deal
//! with registers or commands for a device.

use prelude::v1::*;
use base::*;
use bus::*;
use register::*;
use system::*;

pub mod commands;
pub mod registers;

pub mod device;

pub mod i2c;
pub mod spi;