use periphery_core::prelude::v1::*;
use periphery_core::*;


extern crate spidev;

use self::spidev::{Spidev, SpidevOptions, SpidevTransfer, SPI_MODE_0};

use std::path::Path;

#[derive(Copy, Clone, Debug)]
pub struct SpiBusSettings {
    pub bits_per_word: u8,
    pub max_speed_hz: u32
}

impl Default for SpiBusSettings {
    fn default() -> Self {
        SpiBusSettings {
            bits_per_word: 8,
            max_speed_hz: 20_000
        }
    }
}

#[derive(Clone)]
pub struct LinuxSpiBus<S> where S: SystemApi {
    path: String,
    system_api: S,
    settings: SpiBusSettings
}

impl<S> LinuxSpiBus<S> where S: SystemApi {
    pub fn new(path: &str, system_api: S, settings: SpiBusSettings) -> Result<Self, PeripheryError> {
        let bus = LinuxSpiBus {
            path: path.into(),
            system_api: system_api,
            settings: settings
        };
                
        Ok(bus)
    }

    /*
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
    */
}


impl<S> Bus for LinuxSpiBus<S> where S: SystemApi {
    type SystemApi = S;
    type I2C = I2CBusNotImplemented;
    type Spi = Self;

    fn get_spi(&self) -> Result<Self, PeripheryError> {
		Ok(self.clone())
	}

    fn get_system_api(&self) -> S {
		self.system_api.clone()
	}

    fn get_cli_prefix(&self) -> Result<Cow<str>, PeripheryError> {
        Ok(format!("spi-{}", 1).into())
    }
}

impl<S> SpiBus for LinuxSpiBus<S> where S: SystemApi {
    type DeviceFactory = LinuxSpiBusDeviceFactory;

    fn chip_count(&self) -> Result<SpiDeviceNumber, PeripheryError> {
        Ok(1)
    }

    fn new_spi_device_factory(&self, device_number: SpiDeviceNumber) -> Result<Self::DeviceFactory, PeripheryError> {
        Ok(LinuxSpiBusDeviceFactory {
            path: self.path.clone(),
            settings: self.settings.clone()
        })
    }

    /*
	fn chip_select(&self, device_number: u16, enable: bool) -> Result<(), PeripheryError> {
        panic!("todo");
    }

	fn transmit(&self, send: &[u8], receive: &mut [u8]) -> Result<(), PeripheryError> {
        let spi = self.get_spi_dev()?;
        let mut transfer = SpidevTransfer::read_write(send, receive);
        spi.transfer(&mut transfer)?;
        Ok(())
    }
    */
}


pub struct LinuxSpiBusDeviceFactory {
    path: String,
    settings: SpiBusSettings    
}

impl LinuxSpiBusDeviceFactory {
    fn get_spi_dev(&self) -> Result<Spidev, PeripheryError> {
        let mut spi = Spidev::open(self.path.clone())?;
        let mut options = SpidevOptions::new();
        options.bits_per_word(self.settings.bits_per_word)
               .max_speed_hz(self.settings.max_speed_hz)
               .mode(SPI_MODE_0);
        spi.configure(&options)?;
        Ok(spi)
    }
}

use periphery_core::prelude::v1::device_bus::spi::SpiBusDeviceFactory;

impl SpiBusDeviceFactory for LinuxSpiBusDeviceFactory {
    type DataTransfer = LinuxSpiDeviceBus;

    fn new_spi_device_data_transfer(&self) -> Result<Self::DataTransfer, PeripheryError> {        
        Ok(LinuxSpiDeviceBus {
            dev: self.get_spi_dev()?
        })
    }
}

pub struct LinuxSpiDeviceBus {
    dev: Spidev
}
impl DeviceDataTransfer for LinuxSpiDeviceBus {
    fn transmit(&self, data: &[u8]) -> Result<(), PeripheryError> {
        let mut transfer = SpidevTransfer::write(data);
        self.dev.transfer(&mut transfer)?;
        Ok(())
    }

    fn receive(&self, data: &mut [u8]) -> Result<(), PeripheryError> {
        let mut transfer = SpidevTransfer::read(data);
        self.dev.transfer(&mut transfer)?;
        Ok(())
    }
}
