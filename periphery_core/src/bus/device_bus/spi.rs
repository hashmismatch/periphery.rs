use prelude::v1::*;

pub trait SpiBusDeviceFactory : Send + Sync {
	type DataTransfer : DeviceDataTransfer;

	fn new_spi_device_data_transfer(&self) -> Result<Self::DataTransfer, PeripheryError>;
}

pub struct SpiBusDeviceFactoryNotImplemented;
impl SpiBusDeviceFactory for SpiBusDeviceFactoryNotImplemented {
	type DataTransfer = DeviceDataTransferNotImplemented;

	fn new_spi_device_data_transfer(&self) -> Result<Self::DataTransfer, PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}
}
