use prelude::v1::*;
use base::*;
use bus::*;
//use buspirate::*;
use system::*;

use terminal_cli::*;

pub struct PeripheryBusCliState {
	spi_chip_number_selected: Option<u16>
}

impl PeripheryBusCliState {
	pub fn new<B: Bus>(bus: &B) -> Result<PeripheryBusCliState, PeripheryError> {
		let mut spi_chip_number_selected = None;

		if let Ok(spi) = bus.get_spi() {
			if let Ok(c) = spi.chip_count() {
				if c >= 1 {
					spi_chip_number_selected = Some(1);
				}
			}
		}

		Ok(PeripheryBusCliState {
			spi_chip_number_selected: spi_chip_number_selected
		})
	}
}

pub fn periphery_bus_cli<'a, B: Bus>(state: &mut PeripheryBusCliState, bus: &B, exec: &mut CliExecutor<'a>) {

	let cli_prefix = bus.get_cli_prefix().unwrap_or("bus".into());

	/*********************/

	if let Ok(spi) = bus.get_spi() {

		/*
		{
			let cmd = format!("bus/spi/{}/bp ", cli_prefix);
			if let Some(mut ctx) = exec.command(&cmd) {
				let args: String = ctx.get_args().trim().into();

				if args.len() == 0 {					
					print_buspirate_help(|l| {
						ctx.get_terminal().print_line(l)
					});
				} else {
					spi_buspirate(&args, state, spi, ctx.get_terminal());
				}
			}
		}
		*/

		match (spi.chip_count(), state.spi_chip_number_selected) {
			(Ok(c), Some(ref mut spi_chip_number_selected)) => {
				let cmd = format!("bus/spi/{}/selected_chip", cli_prefix);			
				if let Some(mut ctx) = exec.property(cmd, validate_property_min_max(1, c)) {
					ctx.apply(spi_chip_number_selected);
				}
			},
			(_, _) => ()
		}
	}

	/*********************/

	if let Ok(i2c) = bus.get_i2c() {
		/*
		{
			let cmd = format!("bus/i2c/{}/bp ", cli_prefix);
			if let Some(mut ctx) = exec.command(&cmd) {
				let args: String = ctx.get_args().trim().into();

				if args.len() == 0 {					
					print_buspirate_help(|l| {
						ctx.get_terminal().print_line(l)
					});
				} else {
					i2c_buspirate(&args, state, i2c, ctx.get_terminal());
				}
			}
		}
		*/

		{
			let cmd = format!("bus/i2c/{}/scan", cli_prefix);
			if let Some(mut ctx) = exec.command(&cmd) {
				let devices = i2c.detect_devices();
				ctx.get_terminal().print_line(&format!("Detected devices: {:?}", devices));
			}
		}
	}

}

/*
fn spi_buspirate(args: &str, state: &PeripheryBusCliState, spi: &SpiBus, terminal: &mut CharacterTerminalWriter) -> Result<(), PeripheryError> {
	let operations = parse_buspirate_bus(args);
	if let Ok(ops) = operations {
		terminal.print_line(&format!("Performing: {:?}", ops));

		let mut debug = false;

		let chip_selected = state.spi_chip_number_selected.unwrap_or(1);

		for op in &ops {
			match *op {
				BusOperation::ChipSelect | BusOperation::ChipSelectDebug => {
					try!(spi.chip_select(chip_selected, true));
					terminal.print_line(&format!("Chip {} selected", chip_selected));
				},				
				BusOperation::ChipDeselect | BusOperation::ChipDeselectDebug => {
					try!(spi.chip_select(chip_selected, false));
					terminal.print_line(&format!("Chip {} deselected", chip_selected));
				},
				BusOperation::WriteBytes(ref b) => {
					let mut receive_buffer = vec![0; b.len()];
					try!(spi.transmit(&b, &mut receive_buffer));
					if debug {
						terminal.print_line(&format!("Wrote {} bytes: {:?} - received back: {:?}", b.len(), b, receive_buffer));
					} else {
						terminal.print_line(&format!("Wrote {} bytes: {:?}", b.len(), b));
					}
				},
				BusOperation::ReadBytes(num) => {
					let send_buf = vec![0xFF; num as usize];
					let mut receive_buffer = vec![0; num as usize];
					try!(spi.transmit(&send_buf, &mut receive_buffer));
					terminal.print_line(&format!("Read {} bytes: {:?}", num, receive_buffer));
				}
			}

			if *op == BusOperation::ChipSelectDebug { debug = true; }
			if *op == BusOperation::ChipDeselectDebug { debug = false; }
		}
	} else {
		terminal.print_line("Error parsing!");
	}

	Ok(())
}

#[derive(Debug, PartialEq)]
enum I2CBusOperation {
	WriteBytes(I2CAddress, Vec<u8>),
	ReadBytes(I2CAddress, u8)
}

fn bus_to_i2c(ops: &[BusOperation]) -> Result<Vec<I2CBusOperation>, PeripheryError> {
	let mut addr: Option<I2CAddress> = None;
	let mut v = Vec::new();

	for op in ops {
		match *op {
			BusOperation::WriteBytes(ref b) => {
				if b.len() == 0 {
					return Err(PeripheryError::BusOperationError);
				}

				if let Some(addr) = addr {
					
					//if addr.get_type() == I2CAddressType::Read {
					//	return Err(PeripheryError::BusOperationError);
					//}

					let data_bytes = b.iter().cloned().collect();
					v.push(I2CBusOperation::WriteBytes(addr, data_bytes));
				} else {
					let i2c_addr = I2CAddress::address_7bit(b[0]);
					if i2c_addr.is_reserved() {
						return Err(PeripheryError::BusOperationError);
					}

					addr = Some(i2c_addr);
				}
			},

			BusOperation::ReadBytes(num) => {
				if let Some(addr) = addr {
					v.push(I2CBusOperation::ReadBytes(addr, num));
				} else {
					return Err(PeripheryError::BusOperationError);
				}
			},

			BusOperation::ChipSelect | BusOperation::ChipSelectDebug => {
				addr = None;
			},
			BusOperation::ChipDeselect | BusOperation::ChipDeselectDebug => {
				
			}
		}
	}

	Ok(v)
}

fn i2c_buspirate(args: &str, state: &PeripheryBusCliState, i2c: &I2CBus, terminal: &mut CharacterTerminalWriter) -> Result<(), PeripheryError> {
	let operations = parse_buspirate_bus(args);
	if operations.is_err() {
		terminal.print_line("Error parsing!");
	}
	let operations = operations.unwrap();

	if let Ok(ops) = bus_to_i2c(&operations) {
		terminal.print_line(&format!("Performing: {:?}", ops));

		for op in &ops {
			match *op {				
				I2CBusOperation::WriteBytes(device, ref b) => {
					try!(i2c.write(device, b));
					terminal.print_line(&format!("Wrote {} bytes to {}: {:?}", b.len(), device, b));
				},
				I2CBusOperation::ReadBytes(device, num) => {
					let mut receive_buffer = vec![0; num as usize];
					try!(i2c.read(device, &mut receive_buffer));
					terminal.print_line(&format!("Read {} bytes from {}: {:?}", num, device, receive_buffer));
				}
			}
		}
	} else {
		terminal.print_line("Error parsing to I2C bus operations!");
	}

	Ok(())
}


#[test]
fn test_i2c_bp_parse() {
	let args = "[0xa6 0x32[0xa7 r:2]";

	let p = parse_buspirate_bus(&args).unwrap();
	println!("p: {:?}", p);

	let p = bus_to_i2c(&p).unwrap();

	assert_eq!(&[
		I2CBusOperation::WriteBytes(I2CAddress::address_7bit(0xa6), vec![0x32]),
		I2CBusOperation::ReadBytes(I2CAddress::address_7bit(0xa7), 2)
		], p.as_slice());
}

#[test]
fn test_i2c_bp_parse_2() {
	let args = "[0x77 0xd0 r]";

	let p = parse_buspirate_bus(&args).unwrap();
	println!("p: {:?}", p);

	let p = bus_to_i2c(&p).unwrap();
	println!("p: {:?}", p);
}
*/