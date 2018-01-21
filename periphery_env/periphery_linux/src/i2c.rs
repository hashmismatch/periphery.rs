use periphery_core::prelude::v1::*;
use periphery_core::*;

extern crate i2cdev;

use self::i2cdev::core::*;
use self::i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

use std::path::Path;

#[derive(Clone)]
pub struct LinuxI2CBus<S> where S: SystemApi {
    path: String,
    system_api: S
}

impl<S> LinuxI2CBus<S> where S: SystemApi {
    pub fn new(path: &str, system_api: S) -> Result<Self, PeripheryError> {
        Ok(LinuxI2CBus {
            path: path.into(),
            system_api: system_api
        })
    }

    pub fn detect(system_api: S) -> Vec<Self> {
        let mut ret = vec![];

        for i in 1..16 {
            let path = format!("/dev/i2c-{}", i);
            if Path::new(&path).exists() {
                if let Ok(bus) = Self::new(&path, system_api.clone()) {
                    ret.push(bus);
                }
            }
        }

        ret
    }

    fn get_device_bus(&self, address: I2CAddress) -> Result<LinuxI2CDevice, PeripheryError> {
        if let Ok(dev) = LinuxI2CDevice::new(&self.path, address.get_7bit_address() as u16) {
            Ok(dev)
        } else {
            Err(PeripheryError::BusOperationError)
        }
    }
}

impl<S> Bus for LinuxI2CBus<S> where S: SystemApi {
    type SystemApi = S;
    type I2C = Self;
    type Spi = SpiBusNotImplemented;

    fn get_i2c(&self) -> Result<Self, PeripheryError> {
		Ok(self.clone())
	}

    fn get_system_api(&self) -> S {
		self.system_api.clone()
	}

    fn get_cli_prefix(&self) -> Result<Cow<str>, PeripheryError> {
        Ok(format!("i2c-{}", 1).into())
    }
}

impl<S> I2CBus for LinuxI2CBus<S> where S: SystemApi {
    type DeviceFactory = LinuxI2CBusDeviceFactory;

    fn read(&self, device: I2CAddress, data: &mut [u8]) -> Result<(), PeripheryError> {
        let mut bus = self.get_device_bus(device)?;
        if let Ok(_) = bus.read(data) {
            Ok(())
        } else {
            Err(PeripheryError::BusOperationError)
        }
    }

	fn write(&self, device: I2CAddress, data: &[u8]) -> Result<(), PeripheryError> {
        let mut bus = self.get_device_bus(device)?;
        if let Ok(_) = bus.write(data) {
            Ok(())
        } else {
            Err(PeripheryError::BusOperationError)
        }
    }

    fn read_from_register(&self, device: I2CAddress, address: u8, data: &mut [u8]) -> Result<(), PeripheryError> {
        let mut bus = self.get_device_bus(device)?;

        if let Ok(_) = bus.write(&[address]) {

        } else {
            return Err(PeripheryError::BusOperationError);
        }

        if let Ok(_) = bus.read(data) {
            Ok(())
        } else {
            Err(PeripheryError::BusOperationError)
        }
    }

	fn write_to_register(&self, device: I2CAddress, address: u8, data: &[u8]) -> Result<(), PeripheryError> {
        let mut bus = self.get_device_bus(device)?;

        let mut transfer_data = vec![address];
        transfer_data.extend_from_slice(data);
        
        if let Ok(_) = bus.write(&transfer_data) {
            Ok(())
        } else {
            Err(PeripheryError::BusOperationError)
        }
    }

	fn ping(&self, device: I2CAddress) -> Result<bool, PeripheryError> {
        let mut bus = self.get_device_bus(device)?;
        if let Ok(_) = bus.smbus_read_byte() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn new_device_factory(&self) -> Result<Self::DeviceFactory, PeripheryError> {
        Ok(LinuxI2CBusDeviceFactory {
            path: self.path.clone()
        })
    }
}

/*
pub trait I2CBusDeviceFactory : Send + Sync {
	type Registers : DeviceRegisterBus;
	type Commands : DeviceCommandBus;
	type DataTransfer : DeviceDataTransfer;

	fn new_i2c_device_registers(&self, address: I2CAddress) -> Result<Self::Registers, PeripheryError>;
	fn new_i2c_device_commands(&self, address: I2CAddress) -> Result<Self::Commands, PeripheryError>;
	fn new_i2c_device_data_transfer(&self, address: I2CAddress) -> Result<Self::DataTransfer, PeripheryError>;
}
*/

pub struct LinuxI2CBusDeviceFactory {
    path: String
}

impl I2CBusDeviceFactory for LinuxI2CBusDeviceFactory {
    type Registers = LinuxI2CDeviceBus;
	type Commands = LinuxI2CDeviceBus;
	type DataTransfer = LinuxI2CDeviceBus;

	fn new_i2c_device_registers(&self, address: I2CAddress) -> Result<Self::Registers, PeripheryError> {
        LinuxI2CDeviceBus::new(&self.path, address)
    }

	fn new_i2c_device_commands(&self, address: I2CAddress) -> Result<Self::Commands, PeripheryError> {
        LinuxI2CDeviceBus::new(&self.path, address)
    }

	fn new_i2c_device_data_transfer(&self, address: I2CAddress) -> Result<Self::DataTransfer, PeripheryError> {
        LinuxI2CDeviceBus::new(&self.path, address)
    }
}

use std::sync::Mutex;

pub struct LinuxI2CDeviceBus {
    device: Mutex<LinuxI2CDevice> // external API
}

impl LinuxI2CDeviceBus {
    pub fn new(path: &str, address: I2CAddress) -> Result<Self, PeripheryError> {
        if let Ok(dev) = LinuxI2CDevice::new(path, address.get_7bit_address() as u16) {
            Ok(LinuxI2CDeviceBus { device: Mutex::new(dev) })
        } else {
            Err(PeripheryError::BusOperationError)
        }
    }
}

impl DeviceRegisterBus for LinuxI2CDeviceBus {
    fn read_from_register(&self, register: u8, data: &mut [u8]) -> Result<(), PeripheryError> {
        if let Ok(mut device) = self.device.lock() {
            if let Ok(_) = device.write(&[register]) {

            } else {
                return Err(PeripheryError::BusOperationError);
            }

            if let Ok(_) = device.read(data) {
                Ok(())
            } else {
                Err(PeripheryError::BusOperationError)
            }
        } else {
            Err(PeripheryError::BusOperationError)
        }
    }

    fn write_to_register(&self, register: u8, data: &[u8]) -> Result<(), PeripheryError> {
        if let Ok(mut device) = self.device.lock() {
            let mut transfer_data = vec![register];
            transfer_data.extend_from_slice(data);
            
            if let Ok(_) = device.write(&transfer_data) {
                Ok(())
            } else {
                Err(PeripheryError::BusOperationError)
            }
        } else {
            Err(PeripheryError::BusOperationError)
        }
    }
}

impl DeviceCommandBus for LinuxI2CDeviceBus {
    fn execute_command(&self, data: &[u8]) -> Result<(), PeripheryError> {
        if let Ok(mut device) = self.device.lock() {
            for b in data {
                if let Err(_) = device.write(&[0x00, *b]) {
                    return Err(PeripheryError::BusOperationError);
                }
            }
            Ok(())
        } else {
            Err(PeripheryError::BusOperationError)
        }
    }
}

impl DeviceDataTransfer for LinuxI2CDeviceBus {
    fn transmit(&self, data: &[u8]) -> Result<(), PeripheryError> {
        if let Ok(mut device) = self.device.lock() {
            if let Err(_) = device.write(data) {
                return Err(PeripheryError::BusOperationError);
            }

            Ok(())
        } else {
            Err(PeripheryError::BusOperationError)
        }
    }

    fn receive(&self, data: &mut [u8]) -> Result<(), PeripheryError> {
        if let Ok(mut device) = self.device.lock() {
            if let Err(_) = device.read(data) {
                return Err(PeripheryError::BusOperationError);
            }

            Ok(())
        } else {
            Err(PeripheryError::BusOperationError)
        }
    }
}
