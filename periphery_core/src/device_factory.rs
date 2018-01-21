//! A collection of complicated traits to find and instantiate devices from
//! an instance of a bus. 

use prelude::v1::*;
use base::*;
use bus::*;
use system::*;

pub trait DeviceArguments<B: Bus, A, O> {
    fn new(bus: B, additional: A) -> Result<O, PeripheryError>;
}

pub struct I2CArguments {
    pub i2c_address: I2CAddress
}

pub struct I2CDeviceRegisters<B: Bus> {
    pub system_api: <B as Bus>::SystemApi,
    pub device_bus: <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::Registers
}

impl<B: Bus> DeviceArguments<B, I2CArguments, Self> for I2CDeviceRegisters<B> {
    fn new(bus: B, additional: I2CArguments) -> Result<Self, PeripheryError> {
        let i2c = bus.get_i2c()?;
		let bus_factory = i2c.new_device_factory()?;
        let device_bus = bus_factory.new_i2c_device_registers(additional.i2c_address)?;
        Ok(I2CDeviceRegisters {
            system_api: bus.get_system_api().clone(),
            device_bus: device_bus
        })
    }
}

pub struct I2CDeviceAll<B: Bus> {
    pub system_api: <B as Bus>::SystemApi,
    pub device_registers: <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::Registers,
    pub device_commands: <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::Commands,
    pub device_data: <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::DataTransfer,
}

impl<B: Bus> DeviceArguments<B, I2CArguments, Self> for I2CDeviceAll<B> {
    fn new(bus: B, additional: I2CArguments) -> Result<Self, PeripheryError> {
        let i2c = bus.get_i2c()?;
		let bus_factory = i2c.new_device_factory()?;
        Ok(I2CDeviceAll {
            system_api: bus.get_system_api().clone(),
            device_registers: bus_factory.new_i2c_device_registers(additional.i2c_address)?,
            device_commands: bus_factory.new_i2c_device_commands(additional.i2c_address)?,
            device_data: bus_factory.new_i2c_device_data_transfer(additional.i2c_address)?
        })
    }
}


pub trait DeviceI2CDetection<D: 'static, B, A> : Default
    where D: Device,
          A: DeviceArguments<B, I2CArguments, A>,
          B: Bus
{
	fn get_addresses(&self) -> &[I2CAddress];
	fn new(args: A) -> Result<D, PeripheryError>;


    fn find_device(&self, bus: B) -> Result<D, PeripheryError> {
        let devices = self.find_all_devices(bus)?;
        
        if let Some(first_device) = devices.into_iter().next() {
            Ok(first_device)
        } else {
            Err(PeripheryError::DeviceNotFound)
        }
    }

    fn find_all_devices(&self, bus: B) -> Result<Vec<D>, PeripheryError> {
        let mut ret = vec![];
		
		for i2c_address in self.get_addresses() {

            let i2c_args = I2CArguments {
                i2c_address: *i2c_address
            };

            if let Ok(i2c) = bus.get_i2c() {
                if let Ok(true) = i2c.ping(*i2c_address) {

                } else {
                    continue;
                }
            }

            if let Ok(args) = A::new(bus.clone(), i2c_args) {
                if let Ok(device) = Self::new(args) {
                    ret.push(device);
				}
            }
		}

        if ret.len() > 0 {
            return Ok(ret);
        }

		Err(PeripheryError::DeviceNotFound)
    }
}