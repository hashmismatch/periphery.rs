use prelude::v1::*;
use base::*;
use device::*;
use device_factory::*;
use system::*;
use self::device_bus::i2c::*;

#[derive(Copy, Clone, PartialEq)]
pub struct I2CAddress(u8);

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum I2CAddressType {
	Read,
	Write
}

impl I2CAddress {
	pub fn address_7bit(addr: u8) -> I2CAddress {
		I2CAddress(addr << 1)
	}

	pub fn address_8bit(addr: u8) -> Self {
		I2CAddress(addr)
	}

	pub fn is_reserved(&self) -> bool {
		Self::is_address_reserved(self.0 >> 1)
	}

	/// http://www.i2c-bus.org/addressing/
	pub fn is_address_reserved(addr: u8) -> bool {
		addr == 0 ||                      // General Call
		addr == 1 ||                      // Start Byte
		((addr & 0b1111_1110) >> 1) == 1 ||       // CBUS Addresses
		((addr & 0b1111_1110) >> 1) == 0b10 ||    // Reserved for Different Bus Formats
		((addr & 0b1111_1110) >> 1) == 0b11 ||    // Reserved for future purposes
		((addr & 0b1111_1000) >> 3) == 1 ||       // High-Speed Master Code
		((addr & 0b1111_1000) >> 3) == 0b11110 || // 10-bit Slave Addressing
		((addr & 0b1111_1000) >> 3) == 0b11111    // Reserved for future purposes
	}

	pub fn get_7bit_address(&self) -> u8 {
		self.0 >> 1
	}

	pub fn get_8bit_address_read(&self) -> u8 {
		self.0 | 1
	}

	pub fn get_8bit_address_write(&self) -> u8 {
		self.0 & 0b11111110
	}
}


impl fmt::Display for I2CAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    	write!(f, "0x{:x}", self.get_7bit_address())
    }
}

impl fmt::Debug for I2CAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    	write!(f, "0x{:x}", self.get_7bit_address())
    }
}



pub trait I2CBus : Send + Sync + Clone {
	type DeviceFactory : I2CBusDeviceFactory;

	fn read(&self, device: I2CAddress, data: &mut [u8]) -> Result<(), PeripheryError>;
	fn write(&self, device: I2CAddress, data: &[u8]) -> Result<(), PeripheryError>;

	fn read_from_register(&self, device: I2CAddress, address: u8, data: &mut [u8]) -> Result<(), PeripheryError>;
	fn write_to_register(&self, device: I2CAddress, address: u8, data: &[u8]) -> Result<(), PeripheryError>;
	fn ping(&self, device: I2CAddress) -> Result<bool, PeripheryError>;

	fn detect_devices(&self) -> Vec<I2CAddress> {
		let addresses = get_i2c_scannable_adresses();

		let mut ret = Vec::new();

		for addr in &addresses {
			let r = self.ping(*addr);
			match r {
				Ok(true) => { ret.push(*addr); }
				_ => {}
			}
		}

		ret
	}

	fn new_device_factory(&self) -> Result<Self::DeviceFactory, PeripheryError>;
}

#[derive(Clone)]
pub struct I2CBusNotImplemented;
impl I2CBus for I2CBusNotImplemented {
	type DeviceFactory = I2CBusDeviceFactoryNotImplemented;

	fn read(&self, device: I2CAddress, data: &mut [u8]) -> Result<(), PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}
	fn write(&self, device: I2CAddress, data: &[u8]) -> Result<(), PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}

	fn read_from_register(&self, device: I2CAddress, address: u8, data: &mut [u8]) -> Result<(), PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}
	fn write_to_register(&self, device: I2CAddress, address: u8, data: &[u8]) -> Result<(), PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}

	fn ping(&self, device: I2CAddress) -> Result<bool, PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}

	fn new_device_factory(&self) -> Result<Self::DeviceFactory, PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}
}


pub trait I2CBusScanner {
	fn detect_devices(&self) -> Vec<I2CAddress>;
}

pub fn get_i2c_scannable_adresses() -> Vec<I2CAddress> {
	let mut a = Vec::new();

	for i in 0..255 {
		if !I2CAddress::is_address_reserved(i) {
			a.push(I2CAddress(i));
		}
	}

	a
}


#[cfg(test)]
#[test]
pub fn list_i2c_scan_adresses() {
	let a = get_i2c_scannable_adresses();
	assert_eq!(a.len(), 224);
}
