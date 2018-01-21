use prelude::v1::*;

pub extern crate periphery_core as core;

// devices
pub mod devices {
	pub extern crate apa102;
	//pub extern crate apds_9960;
	pub extern crate bmp180;
	pub extern crate bmp280;
	pub extern crate hmc5883;
	pub extern crate invensense_mpu;
	pub extern crate ms5611;
	pub extern crate sht3x;
	//pub extern crate spi_flash;
	pub extern crate ssd1306;
	pub extern crate fusb302;
}

// features
pub mod features {
	//pub extern crate gesture_detection;
	pub extern crate orientation_detection;
}

use self::core::*;
use self::core::prelude::v1::*;

pub struct DetectedDevices<B> where B: Bus + 'static {
	devices: Vec<DeviceKind<B>>
}

pub struct SensorOnDevice<'a, T: 'a> where T: ?Sized {
	pub device: &'a Device,
	pub sensor: &'a T
}


macro_rules! find_sensor {
	($function: ident, $getter: ident, $T: ty) => {
		pub fn $function(&self, filter: &DeviceFilter) -> Option<SensorOnDevice<$T>> {
			for device in self.get_devices() {
				let device = device.get_device();
				if !filter.matches(device) { continue; }

				if let Some(s) = device.$getter() {
					if let Some(measurement_quality_at_least) = filter.measurement_quality_at_least {
						if s.get_sensor_measurement_quality() < measurement_quality_at_least {
							continue;
						}
					}

					return Some(SensorOnDevice { device: device, sensor: s });
				}
			}

			None
		}
	};
}

impl<B> DetectedDevices<B> where B: Bus + 'static {
	pub fn get_devices(&self) -> &[DeviceKind<B>] {
		&self.devices
	}

	// todo: more filters

	find_sensor!(find_ambient_light_sensor, get_ambient_light_sensor, AmbientLightSensor);
	find_sensor!(find_ambient_temperature_sensor, get_ambient_temperature_sensor, AmbientTemperatureSensor);
	find_sensor!(find_atmospheric_pressure_sensor, get_atmospheric_pressure_sensor, AtmosphericPressureSensor);
	find_sensor!(find_atmospheric_humidity_sensor, get_atmospheric_humidity_sensor, AtmosphericHumiditySensor);	
}

#[derive(Clone, Debug)]
pub struct DeviceFilter {
	pub description_contains: Option<Cow<'static, str>>,
	pub id_contains: Option<Cow<'static, str>>,
	pub measurement_quality_at_least: Option<MeasurementQuality>
}

impl DeviceFilter {
	fn matches(&self, device: &Device) -> bool {
		if let Some(ref description_contains) = self.description_contains {
			let description_contains: &str = &description_contains;
			if !device.description().contains(description_contains) {
				return false;
			}
		}
		
		if let Some(ref id_contains) = self.id_contains {
			let id_contains: &str = &id_contains;
			if !device.id().contains(id_contains) {
				return false;
			}
		}

		return true;
	}
}

impl Default for DeviceFilter {
	fn default() -> Self {
		DeviceFilter {
			description_contains: None,
			id_contains: None,
			measurement_quality_at_least: None
		}
	}
}


pub trait DeviceKindGetImpl<D> {
	fn get_device_impl(&self) -> Option<&D>;
}


pub fn devices_detect_all<B>(bus: B) -> Vec<Box<Device + Send + Sync>> where B: Bus + 'static {
	devices_detect_all_castable(bus).devices.into_iter().map(|d| d.into_boxed()).collect()
}


pub fn devices_detect_all_castable<B>(bus: B) -> DetectedDevices<B> where B: Bus + 'static {
	let mut devices = vec![];

	{
		let f: devices::bmp180::Bmp180Factory = Default::default();
		if let Ok(d) = f.find_all_devices(bus.clone()) {
			for device in d {
				devices.push(device.into());
			}
		}
	}

	{
		let f: devices::bmp280::Bmp280Factory = Default::default();
		if let Ok(d) = f.find_all_devices(bus.clone()) {
			for device in d {
				devices.push(device.into());
			}
		}
	}	

	{
		let f: devices::ssd1306::Ssd1306Factory = Default::default();
		if let Ok(d) = f.find_all_devices(bus.clone()) {
			for device in d {
				devices.push(device.into());
			}
		}
	}

	/*
	{
		let f: devices::apds_9960::Apds9960Factory = Default::default();
		if let Ok(d) = f.find_all_devices(bus.clone()) {
			for device in d {
				devices.push(device.into());
			}
		}
	}
	*/

	{
		let f: devices::sht3x::Sht3xFactory = Default::default();
		if let Ok(d) = f.find_all_devices(bus.clone()) {
			for device in d {
				devices.push(device.into());
			}
		}
	}

	{
		let f: devices::hmc5883::Hmc5883Factory = Default::default();
		if let Ok(d) = f.find_all_devices(bus.clone()) {
			for device in d {
				devices.push(device.into());
			}
		}
	}

	{
		let f: devices::ms5611::Ms5611Factory = Default::default();
		if let Ok(d) = f.find_all_devices(bus.clone()) {
			for device in d {
				devices.push(device.into());
			}
		}
	}

	{
		let f: devices::invensense_mpu::InvensenseMpuFactory = Default::default();
		if let Ok(d) = f.find_all_devices(bus.clone()) {
			for device in d {
				devices.push(device.into());
			}
		}
	}	

	{
		let f: devices::fusb302::Fusb302Factory = Default::default();
		if let Ok(d) = f.find_all_devices(bus.clone()) {
			for device in d {
				devices.push(device.into());
			}
		}
	}		

	DetectedDevices {
		devices: devices
	}
}


macro_rules! devices {
    (
		$(
			id $id: ident : $T: path
		),+
    ) => (


		pub enum DeviceKind<B> where B: Bus {
			$(
				$id($T),
			)*
		}

		impl<B> DeviceKind<B> where B: Bus + 'static {
			pub fn into_boxed(self) -> Box<Device + Send + Sync> {
				match self {
					$(
						DeviceKind::$id(dev) => Box::new(dev),
					)*
				}
			}

			pub fn get_device(&self) -> &Device {
				match self {
					$(
						&DeviceKind::$id(ref dev) => dev,
					)*
				}
			}
		}

		$(
			impl<B> From<$T> for DeviceKind<B> where B: Bus {
				fn from(device: $T) -> Self {
					DeviceKind::$id(device)
				}
			}

						
			impl<B> DeviceKindGetImpl<$T> for DeviceKind<B> where B: Bus {
				fn get_device_impl(&self) -> Option<& $T> {
					match self {
						&DeviceKind::$id(ref d) => Some(d),
						_ => None
					}
				}
			}
		)*

	)
}


devices! {
	id Bmp180I2C: devices::bmp180::Bmp180OnI2CBus<B>,
	id Bmp280I2C: devices::bmp280::Bmp280OnI2CBus<B>,
	id InvensenseMpu: devices::invensense_mpu::InvensenseMpuOnI2CBus<B>,
	id Ms5611I2C: devices::ms5611::Ms5611OnI2CBus<B>,
	id Hmc5883I2C: devices::hmc5883::Hmc5883OnI2CBus<B>,
	//id SpiFlash: devices::spi_flash::SpiFlash<S, B>,
	id Sht3xI2C: devices::sht3x::Sht3xOnI2CBus<B>,
	id Ssd1306I2C: devices::ssd1306::Ssd1306OnI2CBus<B>,
	//id Apds9960I2C: devices::apds_9960::Apds9960OnI2CBus<B>,
	id Fusb302I2C: devices::fusb302::Fusb302OnI2CBus<B>
}
