//! A collection of registers on a particular device.

use prelude::v1::*;
use base::*;
use bus::*;
use register::*;
use system::*;

pub struct RegisterAddress<'a, T, B: 'a> where B: DeviceRegisterBus {
	pub address: u8,
	pub size_bytes: usize,
	pub register_bus: &'a B,
	pub _register_type: PhantomData<T>
}

#[derive(Clone, Debug)]
pub struct RegisterParseError {
	buffer: [u8; 32],
	size: usize
}

impl RegisterParseError {
	pub fn get_buffer(&self) -> &[u8] {
		&self.buffer[..self.size]
	}
}



impl<'a, T, B> RegisterAddress<'a, T, B> where T: Register, B: DeviceRegisterBus {
	pub fn read_into_buffer(&self, buffer: &mut[u8]) -> Result<(), PeripheryError> {
		if buffer.len() != self.size_bytes {
			return Err(PeripheryError::RegisterSizeMismatch);
		}

		try!(self.register_bus.read_from_register(self.address, buffer));

		Ok(())
	}

	pub fn read_with_parse_error_raw(&self) -> Result<T, (Option<RegisterParseError>, PeripheryError)> {
		let mut buff = [0; 32];
		if self.size_bytes > buff.len() {
			return Err((None, PeripheryError::RegisterOversized));
		}		
		let deserialized = {
			let mut register_buffer = &mut buff [..self.size_bytes as usize];
			match self.read_into_buffer(register_buffer) {
				Ok(_) => (),
				Err(e) => {
					return Err((None, e));
				}
			}

			T::from_register_value(register_buffer)
		};
		
		match deserialized {
			Ok(v) => Ok(v),
			Err(e) => {
				Err((Some(RegisterParseError { buffer: buff, size: self.size_bytes }), e))
			}
		}
	}

	pub fn read(&self) -> Result<T, PeripheryError> {
		match self.read_with_parse_error_raw() {
			Ok(v) => Ok(v),
			Err((_, e)) => Err(e)
		}
	}

	pub fn write(&self, value: &T) -> Result<(), PeripheryError> {
		let mut buff = [0; 32];
		if buff.len() < self.size_bytes {
			return Err(PeripheryError::RegisterSizeMismatch);
		}
		let mut buff = &mut buff [0..self.size_bytes as usize];
		try!(value.to_register_value(buff));
		self.register_bus.write_to_register(self.address, buff)
	}

	pub fn write_raw(&self, value: &[u8]) -> Result<(), PeripheryError> {
		if value.len() != self.size_bytes {
			return Err(PeripheryError::RegisterSizeMismatch);
		}
		self.register_bus.write_to_register(self.address, value)
	}

	pub fn modify<F: Fn(&mut T) -> ()>(&self, action: F) -> Result<T, PeripheryError> {
		let mut r = self.read()?;
		action(&mut r);
		self.write(&r)?;
		Ok(r)
	}
}

use terminal_cli::*;

pub trait RegisterAddressCli {
	fn read_to_debug_string(&self, t: &mut CharacterTerminalWriter) -> fmt::Result;
	fn write_from_u8(&self, t: &mut CharacterTerminalWriter, input: &str) -> fmt::Result;
}

impl<'a, T, B> RegisterAddressCli for RegisterAddress<'a, T, B> where T: Register + Debug + Display, B: DeviceRegisterBus {
	fn read_to_debug_string(&self, t: &mut CharacterTerminalWriter) -> fmt::Result {
		match self.read_with_parse_error_raw() {
			Ok(v) => {
				write!(t, "{}\r\n", v)
			},
			Err((Some(parse_error), e)) => {
				write!(t, "Error parsing: {:?} Raw register value: {:?}\r\n", e, parse_error.get_buffer())
			},
			Err((None, e)) => {
				write!(t, "Error reading: {:?}\r\n", e)
			}
		}
	}
	
	fn write_from_u8(&self, t: &mut CharacterTerminalWriter, input: &str) -> fmt::Result {		
		match ::periphery_buspirate_parser::parse_u8_array(input) {
			Ok(a) => {
				if a.len() != self.size_bytes as usize {
					return write!(t, "Expected {} bytes!\r\n", self.size_bytes);
				} else {
					try!(write!(t, "Raw bytes: {:?}.", a));

					match self.register_bus.write_to_register(self.address, &a) {
						Ok(_) => {
							try!(write!(t, " Written.\r\n"));
						},
						Err(e) => {
							try!(write!(t, " Error writing to register: {:?}\r\n", e));
						}
					}

					return Ok(())
				}
			},
			Err(e) => {
				write!(t, "Error parsing byte arguments: {:?}\r\n", e)
			}
		}
	}
}






#[macro_export]
macro_rules! registers {
    (
    	$(#[$chip_docs:meta])*
    	chip $chip: ident
    	{$
    		(
    			$(#[$attr:meta])*
    			register [$address: expr; $size_bytes: expr] => $name: ident : $T: ty
    		),+
    	}

    ) => (

		$(#[$chip_docs:meta])*
		#[derive(Clone)]
		pub struct $chip<'a, B: 'a> where B: DeviceRegisterBus {
			register_bus: &'a B
		}

		impl<'a, B> $chip<'a, B> where B: DeviceRegisterBus {
			#[inline]
			pub fn new(register_bus: &'a B) -> Self {
				$chip {
					register_bus: register_bus
				}
			}

			$(
				$(#[$attr])*
				#[inline]
				pub fn $name(&self) -> registers::RegisterAddress<$T, B> {
					registers::RegisterAddress {
						address: $address,
						size_bytes: $size_bytes,
						register_bus: &self.register_bus,
						_register_type: PhantomData::<$T>
					}
				}
			)+			
		}

		impl<'a, B> Display for $chip<'a, B> where B: DeviceRegisterBus {
			fn fmt(&self, f: &mut Formatter) -> fmt::Result {
				try!(write!(f, "Chip {}\r\n", stringify!($chip)));
				$(
					try!(write!(f, "Register [0x{address:X}; {size_bytes}] => {name}: {ty}\r\n", 
						        address = $address,
						        size_bytes = $size_bytes,
						        name = stringify!($name),
						        ty = stringify!($T)
						));
				)+
				Ok(())
			}
		}
		
		impl<'a, B> RegisterBusCli for $chip<'a, B> where B: DeviceRegisterBus {
			fn registers_cli<'b>(&self, exec: &mut $crate::terminal_cli::PrefixedExecutor) {
    			use $crate::terminal_cli::*;
				use $crate::bus::device_bus::registers::*;
    			
				if let Some(mut ctx) = exec.command(&"list_registers") {
					write!(ctx.get_terminal(), "{}", &self);
				}

				$(
					let name = stringify!($name);

					let read_cmd = format!("register/{}/read", name);
					if let Some(mut ctx) = exec.command(&read_cmd) {
						&self.$name().read_to_debug_string(ctx.get_terminal());
					}
				
					let write_cmd = format!("register/{}/write ", name);
					if let Some(mut ctx) = exec.command(&write_cmd) {
						let args = ctx.get_args().to_string();
						&self.$name().write_from_u8(ctx.get_terminal(), &args);
					}
				)+		
    			
    		}
		}

    )
}

