use prelude::v1::*;
use base::*;

pub trait Register where Self: Sized {
	fn to_register_value(&self, output: &mut [u8]) -> Result<(), PeripheryError>;
	fn from_register_value(value: &[u8]) -> Result<Self, PeripheryError>;
}

use ::packed_struct::prelude::*;

impl<P: PackedStructSlice> Register for P {
	fn to_register_value(&self, output: &mut [u8]) -> Result<(), PeripheryError> {
		self.pack_to_slice(output)?;
		Ok(())
	}

	fn from_register_value(value: &[u8]) -> Result<Self, PeripheryError> {
		Ok(Self::unpack_from_slice(value)?)
	}
}

#[derive(Copy, Clone, Default)]
pub struct EmptyReg;
impl Register for EmptyReg {
	fn to_register_value(&self, output: &mut [u8]) -> Result<(), PeripheryError> {
		Ok(())
	}

	fn from_register_value(value: &[u8]) -> Result<Self, PeripheryError> {
		Ok(EmptyReg)
	}
}

impl Debug for EmptyReg {
	fn fmt(&self, _: &mut Formatter) -> fmt::Result {
		Ok(())
	}
}

impl Display for EmptyReg {
	fn fmt(&self, _: &mut Formatter) -> fmt::Result {
		Ok(())
	}
}

// todo: move into a separate crate, for ease of use for no_std

/*
#[cfg(test)]
mod test {
	use super::*;
	use prelude::v1::*;
	use base::*;
	use packed_struct::*;


	#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_compact)]	
	#[repr(u8)]
	#[packed_bit_width="4"]
	pub enum DataRate {
		/// No output data is produced.
		PowerDown = 0,
		Rate_3_125Hz = 1,
		Rate_6_25Hz = 2,
		Rate_12_5Hz = 3,
		Rate_25Hz = 4,
		Rate_50Hz = 5,
		Rate_100Hz = 6,
		Rate_400Hz = 7,
		Rate_800Hz = 8,
		Rate_1600Hz = 9
	}

	// Imaginary register, for test purposes only
	#[derive(PackableBytes, Copy, Clone, Debug, PartialEq)]
	/// Control register 4
	pub struct ControlRegister4 {
		/// Data rate selection
		#[pack_bits="1..5"]
		pub output_data_rate: DataRate, 		
		/// Z-axis enabled?
		#[pack_bits="6..7"]
		pub z_axis_enabled: bool
	}

	#[test]
	fn test_reg() {
		let r = ControlRegister4 {
			output_data_rate: DataRate::Rate_6_25Hz,
			z_axis_enabled: true
		};

		let b = r.pack();
		assert_eq!([0b00010010], b);
		let mut d = [0];
		let b_reg = r.to_register_value(&mut d).unwrap();
		assert_eq!(&b, &d);

		let unpacked = ControlRegister4::unpack(&b).unwrap();
		assert_eq!(&r, &unpacked);

		let unpacked = ControlRegister4::from_register_value(&d).unwrap();
		assert_eq!(&r, &unpacked);
	}
}

*/