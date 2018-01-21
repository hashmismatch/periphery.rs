//! Basic structures for the framework

use prelude::v1::*;
use packed_struct::PackingError;

/// The base error type for the framework
#[derive(Debug, Clone)]
pub enum PeripheryError {
	MissingSystemSleep,
	MissingI2CBus,
	Unknown,	
	Timeout,
	ReadError,
	ReadParseError,	
	BufferLengthError,
	UnsupportedFieldValue,
	RegisterSizeMismatch,
	RegisterOversized,
	WriteError,
	ParseError,
	DataNotAvailable,
	CalculationError,
	MeasurementOverflow,
	MeasurementNotReady,

	BusOperationError,

	LockingError,
	
	ExternalError(i16),
	NotImplemented,

	ReadinessTimeout,
	UnsupportedDevice,
	DeviceNotFound,

	BusPirateParseError,

	CrcMismatch { expected: u16, calculated: u16 },

	PackingError(PackingError),

	#[cfg(feature = "std")]
	StdIoError { description: String },
	SensorError { sensor: Cow<'static, str>, error: Cow<'static, str> }
}


#[cfg(feature = "std")]
impl From<::std::io::Error> for PeripheryError {
	fn from(err: ::std::io::Error) -> Self {
		use ::std::error::Error;

		PeripheryError::StdIoError { description: err.description().into() }
	}
}

impl From<PackingError> for PeripheryError {
	fn from(err: PackingError) -> Self {		
		PeripheryError::PackingError(err)
	}
}
