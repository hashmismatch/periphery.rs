extern crate periphery_flex;

use periphery_flex::*;
use periphery_flex::core::*;
use periphery_flex::core::prelude::v1::*;

use std::thread;
use std::borrow::Cow;

#[derive(Copy, Clone)]
struct StdSystemApi;
impl SystemApi for StdSystemApi {
    fn get_sleep(&self) -> Result<&SystemApiSleep, PeripheryError> {
		Ok(self)
	}

	fn get_debug(&self) -> Result<&SystemApiDebug, PeripheryError> {
		Ok(self)
	}
}

impl SystemApiSleep for StdSystemApi {
    fn sleep_ms(&self, ms: u32) {
        thread::sleep_ms(ms);
    }
}

impl SystemApiDebug for StdSystemApi {
    fn debug(&self, line: &str) {
        println!("Bus debug: {}", line);
    }
}

#[derive(Copy, Clone)]
struct MockedBus { system_api: StdSystemApi }
impl MockedBus {
    pub fn new() -> Self {
        MockedBus {
            system_api: StdSystemApi
        }
    }
}
impl Bus for MockedBus {
    type SystemApi = StdSystemApi;
    type I2C = Self;
    type Spi = SpiBusNotImplemented;

    fn get_cli_prefix(&self) -> Result<Cow<str>, PeripheryError> {
        Ok("bus".into())
    }
    
    fn get_i2c(&self) -> Result<Self::I2C, PeripheryError> {
		Ok(*self)
	}

    fn get_system_api(&self) -> StdSystemApi {
		self.system_api
	}
}

use periphery_flex::core::prelude::v1::device::DeviceRegisterBusNotImplemented;

impl I2CBus for MockedBus {
    type DeviceFactory = MockBusI2CDeviceFactory;

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
        if device.get_7bit_address() == 0x76 {
            return Ok(true);
        }

        Err(PeripheryError::NotImplemented)
    }

    fn new_device_factory(&self) -> Result<Self::DeviceFactory, PeripheryError> {
        Ok(MockBusI2CDeviceFactory)
    }
}

pub struct MockBusI2CDeviceFactory;
impl I2CBusDeviceFactory for MockBusI2CDeviceFactory {
    type Registers = I2CDeviceRegisterBus;
    type Commands = DeviceCommandBusNotImplemented;
    type DataTransfer = DeviceDataTransferNotImplemented;

    fn new_i2c_device_registers(&self, address: I2CAddress) -> Result<Self::Registers, PeripheryError> {
        Ok(I2CDeviceRegisterBus(address))
    }

    fn new_i2c_device_commands(&self, address: I2CAddress) -> Result<Self::Commands, PeripheryError> {
        Err(PeripheryError::NotImplemented)
    }

	fn new_i2c_device_data_transfer(&self, address: I2CAddress) -> Result<Self::DataTransfer, PeripheryError> {
        Err(PeripheryError::NotImplemented)
    }
}

#[derive(Clone)]
pub struct I2CDeviceRegisterBus(I2CAddress);
impl DeviceRegisterBus for I2CDeviceRegisterBus {
    fn read_from_register(&self, register: u8, data: &mut [u8]) -> Result<(), PeripheryError> {
        if self.0.get_7bit_address() == 0x76 {
            match (register, data.len()) {
                (0xD0, 1) => {
                    data[0] = 0x58; return Ok(());
                },
                (_, _) => ()
            }
        }

        Err(PeripheryError::BusOperationError)
    }

    fn write_to_register(&self, register: u8, data: &[u8]) -> Result<(), PeripheryError> {
        Err(PeripheryError::BusOperationError)
    }
}



#[test]
fn test_mocked_bmp280() {
    use periphery_flex::devices::bmp280::*;

    let system_api = StdSystemApi;
    let bus = MockedBus::new();

    {
        let factory: Bmp280Factory = Default::default();
        let bmp280 = factory.find_device(bus).unwrap();
    }

    {        
        let detected = devices_detect_all_castable(bus);
        assert_eq!(detected.get_devices().len(), 1);

        let bmp280_device: &Bmp280<_> = detected.get_devices()[0].get_device_impl().unwrap();
        assert!(bmp280_device.description().contains("BMP280"));

        // will fail
        bmp280_device.read_calibration_coefficients();
    }
}
