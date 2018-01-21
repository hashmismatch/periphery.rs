use prelude::v1::*;
use base::*;
use device::*;
use system::*;

/// Environment for a device to be created from. Encapsulates a particular
/// bus with devices connected to them.
pub trait Bus : Clone + Send + Sync {
	/// The required system environment interface
	type SystemApi : SystemApi;
	/// Optional I2C implementation
	type I2C : I2CBus;
	/// Optional Spi implementation
	type Spi : SpiBus;

	/// Get the optional I2C implementation
	fn get_i2c(&self) -> Result<Self::I2C, PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}

	/// Get the optional Spi implementation
	fn get_spi(&self) -> Result<Self::Spi, PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}

	/// Get the system API
	fn get_system_api(&self) -> Self::SystemApi;

	/// Get the prefix for the terminal commands related to this bus
	fn get_cli_prefix(&self) -> Result<Cow<str>, PeripheryError>;
}
