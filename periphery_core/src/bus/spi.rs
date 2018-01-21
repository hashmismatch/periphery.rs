use prelude::v1::*;
use base::*;
use device::*;
use system::*;

use bus::device_bus::spi::*;

pub type SpiDeviceNumber = u16;

pub trait SpiBus : Send + Sync {
	type DeviceFactory : SpiBusDeviceFactory;

	fn chip_count(&self) -> Result<SpiDeviceNumber, PeripheryError>;

	fn new_spi_device_factory(&self, device_number: SpiDeviceNumber) -> Result<Self::DeviceFactory, PeripheryError>;

	/*	
	fn chip_select(&self, device_number: SpiDeviceNumber, enable: bool) -> Result<(), PeripheryError>;
	fn transmit(&self, send: &[u8], receive: &mut [u8]) -> Result<(), PeripheryError>;
	*/
}

pub struct SpiBusNotImplemented;
impl SpiBus for SpiBusNotImplemented {
	type DeviceFactory = SpiBusDeviceFactoryNotImplemented;
	
	fn chip_count(&self) -> Result<SpiDeviceNumber, PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}

	fn new_spi_device_factory(&self, device_number: SpiDeviceNumber) -> Result<Self::DeviceFactory, PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}
}

/*
pub struct SpiBusChipSelect<'a> {
	spi: &'a SpiBus,
	device_number: u16
}

impl<'a> SpiBusChipSelect<'a> {
	pub fn lock(bus: &'a SpiBus, device_number: SpiDeviceNumber) -> Result<SpiBusChipSelect<'a>, PeripheryError> {
		try!(bus.chip_select(device_number, true));

		Ok(SpiBusChipSelect {
			spi: bus,
			device_number: device_number
		})
	}
}

impl<'a> Drop for SpiBusChipSelect<'a> {
    fn drop(&mut self) { 
    	self.spi.chip_select(self.device_number, false);
    }
}
*/
