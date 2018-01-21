#[macro_use]
extern crate periphery_core;

extern crate packed_struct;

#[macro_use]
extern crate packed_struct_codegen;

use periphery_core::*;
use periphery_core::prelude::v1::*;
use periphery_core::terminal_cli::*;


pub type Sht3xOnI2CBus<B> = Sht3x<<B as Bus>::SystemApi, <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::DataTransfer>;


#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(endian="msb", size_bytes="6")]
pub struct Measurement {
    temperature: u16,
    temperature_crc: u8,
    humidity: u16,
    humidity_crc: u8
}


#[derive(Clone, Copy)]
pub struct Sht3xFactory {
    addresses: [I2CAddress; 2]
}

impl Default for Sht3xFactory {
    fn default() -> Self {
        Sht3xFactory {
            addresses: [
                I2CAddress::address_7bit(0x44),
                I2CAddress::address_7bit(0x45)
            ]
        }
    }
}

impl<B> DeviceI2CDetection<Sht3xOnI2CBus<B>, B, I2CDeviceAll<B>> for Sht3xFactory 
    where B: Bus + 'static,
{
	fn get_addresses(&self) -> &[I2CAddress] {
        &self.addresses
    }

	fn new(args: I2CDeviceAll<B>) -> Result<Sht3xOnI2CBus<B>, PeripheryError> {
        let sensor = Sht3x {
            system: args.system_api,
            bus: args.device_data
        };

        sensor.bus.transmit(&[0x30, 0x41])?;
        let mut r = [0; 3];
        sensor.bus.receive(&mut r)?;
        if r[0] != 0 || r[1] != 0 {
            return Err(PeripheryError::DeviceNotFound);
        }
        
        Ok(sensor)
    }
}


#[derive(Clone)]
pub struct Sht3x<S, B> where S: SystemApi, B: DeviceDataTransfer {
    system: S,
    bus: B
}

impl<S, B> Sht3x<S, B> where S: SystemApi, B: DeviceDataTransfer {
    fn get_measurement_blocking(&self) -> Result<Measurement, PeripheryError> {
        use packed_struct::PackedStruct;
    
        self.bus.transmit(&[0x24, 0x00])?;

        self.system.get_sleep()?.sleep_ms(500);

        let mut buffer = [0; 6];
        self.bus.receive(&mut buffer)?;

        Ok(Measurement::unpack(&buffer)?)
    }
}


impl<S, B> Device for Sht3x<S, B> where S: SystemApi, B: DeviceDataTransfer {
	fn get_ambient_temperature_sensor(&self) -> Option<&AmbientTemperatureSensor> {
		Some(self)
	}

	fn get_atmospheric_humidity_sensor(&self) -> Option<&AtmosphericHumiditySensor> {
		Some(self)
	}	

	fn description(&self) -> Cow<str> {
		"SHT3x humidity and temperature sensor".into()
	}
	
	fn id(&self) -> Cow<str> {
		"sht3x".into()
	}
}

impl<S, B> AmbientTemperatureSensor for Sht3x<S, B> where S: SystemApi, B: DeviceDataTransfer {
	fn get_ambient_temperature(&self) -> Result<AmbientTemperature, PeripheryError> {
		let m = self.get_measurement_blocking()?;
        let t = -45.0 + 175.0 * (m.temperature as f32 / (0xFFFF as f32));
        let t = AmbientTemperature::from_temperature(Temperature::from_degrees_celsius(t));
        Ok(t)
	}
}


impl<S, B> AtmosphericHumiditySensor for Sht3x<S, B> where S: SystemApi, B: DeviceDataTransfer {
	fn get_relative_atmospheric_humidity(&self) -> Result<RelativeHumidity, PeripheryError> {
		let m = self.get_measurement_blocking()?;
        let h = 100.0 * (m.temperature as f32 / (0xFFFF as f32));
        Ok(RelativeHumidity::from_percentage(Percentage::from_percentage(h)))
	}
}