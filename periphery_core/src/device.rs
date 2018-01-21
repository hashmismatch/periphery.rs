//! The basic trait that allows interaction with any detected device. Abstracts away
//! any implementation specific interfaces, exposes only the functionality of a particular device.

use prelude::v1::*;
use base::*;
use bus::*;
use units::*;
use system::*;
use device_storage::*;

use terminal_cli::*;


pub struct DeviceBusCli<'a> {
	registers: Option<Box<RegisterBusCli + 'a>>,
	commands: Option<Box<BusCommandsCli + 'a>>
}

impl<'a> DeviceBusCli<'a> {
	pub fn new() -> Self {
		DeviceBusCli {
			registers: None,
			commands: None
		}
	}

	pub fn with_registers<R: RegisterBusCli + 'a>(&mut self, registers: R) -> &mut Self {
		self.registers = Some(Box::new(registers));
		self
	}

	pub fn with_commands<C: BusCommandsCli + 'a>(&mut self, commands: C) -> &mut Self {
		self.commands = Some(Box::new(commands));
		self
	}
}



pub trait RegisterBusCli {
	fn registers_cli<'b>(&self, exec: &mut ::terminal_cli::PrefixedExecutor);	
}

pub trait BusCommandsCli {
	fn commands_cli<'b>(&self, exec: &mut ::terminal_cli::PrefixedExecutor);	
}


pub trait Device: Send + Sync {
	fn get_ambient_light_sensor(&self) -> Option<&AmbientLightSensor> {
		None
	}
	fn get_ambient_temperature_sensor(&self) -> Option<&AmbientTemperatureSensor> {
		None
	}
	fn get_atmospheric_pressure_sensor(&self) -> Option<&AtmosphericPressureSensor> {
		None
	}
	fn get_atmospheric_humidity_sensor(&self) -> Option<&AtmosphericHumiditySensor> {
		None
	}
	fn get_acceleration_3_sensor(&self) -> Option<&Acceleration3Sensor> {
		None
	}
	fn get_magnetic_field_3_sensor(&self) -> Option<&MagneticField3Sensor> {
		None
	}
	fn get_angular_speed_3_sensor(&self) -> Option<&AngularSpeed3Sensor> {
		None
	}
	fn get_storage_device(&self) -> Option<&StorageDevice> {
		None
	}

	fn get_cli(&self) -> Option<&DeviceCli> {
		None
	}
	
	fn get_registers_cli(&self) -> Option<DeviceBusCli> {
		None
	}

	fn get_data_streams(&self) -> Option<&DataStreams> {
		None
	}

	fn execute_cli<'a>(&self, exec: &mut CliExecutor) {
		if let Some(ref mut exec) = exec.with_prefix(&format!("{}/", self.id())) {
			if let Some(als) = self.get_ambient_light_sensor() {
				if let Some(mut ctx) = exec.command(&"ambient_light/get") {
					match als.get_ambient_light() {
						Ok(t) => ctx.get_terminal().print_line(&format!("{}", t)),
						Err(e) => ctx.get_terminal().print_line(&format!("Error reading ambient light: {:?}", e))
					}
				}
			}

			if let Some(temp) = self.get_ambient_temperature_sensor() {
				if let Some(mut ctx) = exec.command(&"ambient_temperature/get") {
					match temp.get_ambient_temperature() {
						Ok(t) => ctx.get_terminal().print_line(&format!("{}", t)),
						Err(e) => ctx.get_terminal().print_line(&format!("Error reading ambient temperature: {:?}", e))
					}
				}
			}

			if let Some(a3) = self.get_acceleration_3_sensor() {
				if let Some(mut ctx) = exec.command(&"acceleration_3/get") {
					match a3.get_acceleration_3() {
						Ok(t) => ctx.get_terminal().print_line(&format!("{}", t)),
						Err(e) => ctx.get_terminal().print_line(&format!("Error reading acceleration data: {:?}", e))
					}
				}
			}

			if let Some(pressure) = self.get_atmospheric_pressure_sensor() {
				if let Some(mut ctx) = exec.command(&"atmospheric_pressure/get") {
					match pressure.get_atmospheric_pressure() {
						Ok(t) => ctx.get_terminal().print_line(&format!("{}", t)),
						Err(e) => ctx.get_terminal().print_line(&format!("Error reading pressure data: {:?}", e))
					}
				}
			}

			if let Some(humidity) = self.get_atmospheric_humidity_sensor() {
				if let Some(mut ctx) = exec.command(&"atmospheric_humidity/get") {
					match humidity.get_relative_atmospheric_humidity() {
						Ok(t) => ctx.get_terminal().print_line(&format!("{}", t)),
						Err(e) => ctx.get_terminal().print_line(&format!("Error reading humidity data: {:?}", e))
					}
				}
			}

			if let Some(m3) = self.get_magnetic_field_3_sensor() {
				if let Some(mut ctx) = exec.command(&"magnetic_field_3/get") {
					match m3.get_magnetic_field_3() {
						Ok(t) => ctx.get_terminal().print_line(&format!("{}", t)),
						Err(e) => ctx.get_terminal().print_line(&format!("Error reading magnetic field data: {:?}", e))
					}
				}
			}

			if let Some(g3) = self.get_angular_speed_3_sensor() {
				if let Some(mut ctx) = exec.command(&"angular_speed_3/get") {
					match g3.get_angular_speed_3() {
						Ok(t) => ctx.get_terminal().print_line(&format!("{}", t)),
						Err(e) => ctx.get_terminal().print_line(&format!("Error reading angular speed data: {:?}", e))
					}
				}
			}

			if let Some(cli) = self.get_registers_cli() {
				if let Some(registers_cli) = cli.registers {				
					registers_cli.registers_cli(exec);
				}

				if let Some(commands_cli) = cli.commands {
					commands_cli.commands_cli(exec);
				}
			}

			if let Some(cli) = self.get_cli() {
				cli.execute_cli(exec);
			}
		}
	}

	fn init_after_detection(&self) -> Result<bool, PeripheryError> {
		Ok(false)
	}

	fn description(&self) -> Cow<str>;
	fn id(&self) -> Cow<str>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MeasurementQuality {
	Low = 50,
	Normal = 100,
	High = 150
}

macro_rules! device_sensor_fn {
	() => {
		fn get_sensor_measurement_quality(&self) -> MeasurementQuality {
			MeasurementQuality::Normal
		}
	};
}

pub trait AmbientLightSensor {
	fn get_ambient_light(&self) -> Result<Illuminance, PeripheryError>;

	device_sensor_fn!();
}

pub trait AmbientTemperatureSensor {
	fn get_ambient_temperature(&self) -> Result<AmbientTemperature, PeripheryError>;

	device_sensor_fn!();
}

pub trait AtmosphericPressureSensor {
	fn get_atmospheric_pressure(&self) -> Result<AtmosphericPressure, PeripheryError>;

	device_sensor_fn!();
}

pub trait AtmosphericHumiditySensor {
	fn get_relative_atmospheric_humidity(&self) -> Result<RelativeHumidity, PeripheryError>;

	device_sensor_fn!();
}

pub trait Acceleration3Sensor {
	fn get_acceleration_3(&self) -> Result<Acceleration3, PeripheryError>;

	device_sensor_fn!();
}

pub trait MagneticField3Sensor {
	fn get_magnetic_field_3(&self) -> Result<MagneticField3, PeripheryError>;

	device_sensor_fn!();
}

pub trait AngularSpeed3Sensor {
	fn get_angular_speed_3(&self) -> Result<AngularSpeed3, PeripheryError>;

	device_sensor_fn!();
}

pub trait DeviceCli {
	fn execute_cli(&self, exec: &mut PrefixedExecutor);
}

pub trait DataStreams {
	fn get_stream_infos(&self) -> Vec<DataStream>;
	fn get_poller(&self, stream: DataStreamId) -> Result<Box<DataStreamPoller + Send + Sync>, PeripheryError>;
}

pub trait DataStreamPoller {
	fn get_info(&self) -> DataStream;
	fn poll(&mut self) -> Result<Vec<DataStreamPolled>, PeripheryError>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct DataStreamId(pub usize);

#[derive(Debug, Clone)]
pub struct DataStream {
	pub id: DataStreamId,
	pub cli_id: Cow<'static, str>,
	pub description: Cow<'static, str>,
	pub poll_every_ms: usize,
	pub labels: Vec<Cow<'static, str>>
}

#[derive(Debug)]
pub enum DataStreamPolled {
	F32 { data: Vec<f32> },
	Strings { data: Vec<Cow<'static, str>> }
}
