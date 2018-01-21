//! A collection of commands for a particular device.

use prelude::v1::*;
use base::*;
use bus::*;
use register::*;
use system::*;

pub struct ChipCommand<'a, T, B: 'a> where B: DeviceCommandBus {
	pub cmd: u8,
	pub arg_size_bytes: usize,
	pub command_bus: &'a B,
	pub _arg_type: PhantomData<T>
}

impl<'a, T, B: 'a> ChipCommand<'a, T, B> where B: DeviceCommandBus, T: Default {
	pub fn execute(&self) -> Result<(), PeripheryError> {
		self.command_bus.execute_command(&[self.cmd])

		/*

		match rb.chip {
			ChipOnBus::I2C { address } => {
				let i2c = rb.bus.get_i2c()?;

				//try!(i2c.read_from_register(address, register, data));
				i2c.write(address, &[0x00, self.address])?;

				return Ok(());
			},
			ChipOnBus::Spi { chip_number } => {
				panic!("todo spi");
			}
		}
		*/
	}
}

impl<'a, T, B: 'a> ChipCommand<'a, T, B> where B: DeviceCommandBus, T: Register {
	/*
	pub fn execute_args_from_u8(&self, t: &mut CharacterTerminalWriter, input: &str) -> fmt::Result {		
		match ::buspirate::parse_u8_array(input) {
			Ok(a) => {
				if a.len() != self.arg_size_bytes as usize {
					return write!(t, "Expected {} bytes, parsed {:?}!\r\n", self.arg_size_bytes, a);
				} else {
					try!(write!(t, "Raw bytes: {:?}.", a));

					match T::from_register_value(&a) {
						Ok(arg) => {
							try!(write!(t, " Parsed argument: {:?} ", a));

							match self.execute_args(arg) {
								Ok(_) => {
									try!(write!(t, " Executed.\r\n"));
								},
								Err(e) => {
									try!(write!(t, " Error executing: {:?}\r\n", e));
								}
							}
						},
						Err(e) => {
							try!(write!(t, " Error parsing value: {:?}\r\n", e));
						}
					}

					return Ok(());
				}
			},
			Err(e) => {
				write!(t, "Error parsing byte arguments: {:?}\r\n", e)
			}
		}
	}
	*/

	pub fn execute_args(&self, args: T) -> Result<(), PeripheryError> {
		let mut buf = vec![0; self.arg_size_bytes + 1];
		buf[0] = self.cmd;
		if self.arg_size_bytes > 0 {
			let mut buf = &mut buf[1..];
			args.to_register_value(buf)?;
		}
		
		self.command_bus.execute_command(&buf)
		/*
		let rb = self.register_bus;

		match rb.chip {
			ChipOnBus::I2C { address } => {
				let i2c = rb.bus.get_i2c()?;

				//try!(i2c.read_from_register(address, register, data));
				i2c.write(address, &[0x00, self.address])?;

				//i2c.write(address, &[0x80])?;
				{
					let mut buff = [0; 32];
					if buff.len() < self.arg_size_bytes {
						return Err(PeripheryError::RegisterSizeMismatch);
					}
					let mut buff = &mut buff [..self.arg_size_bytes as usize];
					try!(args.to_register_value(buff));

					for b in buff {
						i2c.write(address, &[0x00, *b])?;
					}
				}
				/*
				{
					let mut buff = [0; 33];
					buff[0] = 0x80;

					if buff.len() - 1 < self.arg_size_bytes {
						return Err(PeripheryError::RegisterSizeMismatch);
					}
					let mut buff = &mut buff [1..(self.arg_size_bytes + 1) as usize];
					try!(args.to_register_value(buff));
					i2c.write(address, buff)?;
				}
				*/

				return Ok(());
			},
			ChipOnBus::Spi { chip_number } => {
				panic!("todo spi");
			}
		}
		*/
	}
	
}

#[macro_export]
macro_rules! commands {
    (
    	$(#[$chip_docs:meta])*
    	chip $chip: ident
    	{$
    		(
    			$(#[$attr:meta])*
    			command [$address: expr; $size_bytes: expr] => $name: ident : $T: ty
    		),+
    	}

    ) => (

		$(#[$chip_docs:meta])*
		#[derive(Clone)]
		pub struct $chip<'a, B: 'a> where B: DeviceCommandBus {
			command_bus: &'a B
		}

		impl<'a, B> $chip<'a, B> where B: DeviceCommandBus {
			#[inline]
			pub fn new(command_bus: &'a B) -> Self {
				$chip {
					command_bus: command_bus
				}
			}

			$(
				$(#[$attr])*
				#[inline]				
				pub fn $name(&self) -> ChipCommand<$T, B> {
					ChipCommand {
						cmd: $address,
						arg_size_bytes: $size_bytes,
						command_bus: &self.command_bus,
						_arg_type: PhantomData::<$T>
					}
				}
			)+			
		}


		impl<'a, B> BusCommandsCli for $chip<'a, B> where B: DeviceCommandBus {
			fn commands_cli<'b>(&self, exec: &mut periphery_core::terminal_cli::PrefixedExecutor) {
    			//use $crate::base::*;
    			//use $crate::buspirate::*;
				use periphery_core::terminal_cli::*;
    			
				/*
				let cmd = format!("{}/list_commands", prefix);
				if let Some(mut ctx) = exec.command(&cmd) {
					write!(ctx.get_terminal(), "{}", &self);
				}
				*/

				$(
					let name = stringify!($name);

					let exec_cmd = format!("command/{}/execute", name);
					if let Some(mut ctx) = exec.command(&exec_cmd) {
						let res = self.$name().execute();
						match res {
							Ok(_) => { write!(ctx.get_terminal(), "Command executed.\r\n"); }
							Err(e) => { write!(ctx.get_terminal(), "Error: {:?}", e); }
						};
					}

					/*
					let exec_cmd_args = format!("{}/command/{}/execute_args ", prefix, name);
					if let Some(mut ctx) = exec.command(&exec_cmd_args) {
						let args = ctx.get_args().to_string();
						let args = args.trim();
						let res = self.$name().execute_args_from_u8(ctx.get_terminal(), &args);
						match res {
							Ok(_) => { write!(ctx.get_terminal(), "Command executed.\r\n"); }
							Err(e) => { write!(ctx.get_terminal(), "Error: {:?}", e); }
						};
					}
					*/

					/*
					let write_cmd = format!("{}/commands/execute_args ", prefix, name);
					if let Some(mut ctx) = exec.command(&write_cmd) {
						let args = ctx.get_args().to_string();
						&self.$name().write_from_u8(ctx.get_terminal(), &args);
					}
					*/
				)+

				/*

				match () {
					#[not(cfg(feature="debug_registers"))]
					() => (),
					#[cfg(feature="debug_registers")]
					() => {
						
					}
				} 
				*/   			
    			
    		}
		}		

    )
}
