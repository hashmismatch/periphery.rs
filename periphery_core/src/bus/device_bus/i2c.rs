use prelude::v1::*;

pub trait I2CBusDeviceFactory : Send + Sync {
	type Registers : DeviceRegisterBus;
	type Commands : DeviceCommandBus;
	type DataTransfer : DeviceDataTransfer;

	fn new_i2c_device_registers(&self, address: I2CAddress) -> Result<Self::Registers, PeripheryError>;
	fn new_i2c_device_commands(&self, address: I2CAddress) -> Result<Self::Commands, PeripheryError>;
	fn new_i2c_device_data_transfer(&self, address: I2CAddress) -> Result<Self::DataTransfer, PeripheryError>;
}

pub struct I2CBusDeviceFactoryNotImplemented;
impl I2CBusDeviceFactory for I2CBusDeviceFactoryNotImplemented {
	type Registers = DeviceRegisterBusNotImplemented;
	type Commands = DeviceCommandBusNotImplemented;
	type DataTransfer = DeviceDataTransferNotImplemented;

	fn new_i2c_device_registers(&self, address: I2CAddress) -> Result<Self::Registers, PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}

	fn new_i2c_device_commands(&self, address: I2CAddress) -> Result<Self::Commands, PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}

	fn new_i2c_device_data_transfer(&self, address: I2CAddress) -> Result<Self::DataTransfer, PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}
}
