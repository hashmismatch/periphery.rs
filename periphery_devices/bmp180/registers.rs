use periphery_core::*;
use periphery_core::prelude::v1::*;

use packed_struct::*;

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="2", endian="msb")]
pub struct MeasurementValueMsb {
    pub value: u16
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum ConversionStatus {
    Running = 1,
    Complete = 0
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum PressureOversamplingRatio {
    Times1 = 0,
    Times2 = 0b01,
    Times4 = 0b10,
    Times8 = 0b11
}

impl PressureOversamplingRatio {
	pub fn get_required_ms_wait_after_measurement(&self) -> u32 {
		match *self {
			PressureOversamplingRatio::Times1 => 5,
			PressureOversamplingRatio::Times2 => 8,
			PressureOversamplingRatio::Times4 => 14,
			PressureOversamplingRatio::Times8 => 26
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum MeasurementType {
    Temperature = 0b01110,
    Pressure = 0b10100
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(bit_numbering="msb0")]
pub struct MeasurementControlRegister {
    #[packed_field(bits="0..1", ty="enum")]
    pub oss: PressureOversamplingRatio,
    #[packed_field(bits="2", ty="enum")]
    pub sco: ConversionStatus,
    #[packed_field(bits="3..7", ty="enum")]
    pub measurement: MeasurementType
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(bit_numbering="msb0")]
pub struct ResetRegister {
    #[packed_field(bits="0..7", ty="enum")]
    pub state: ResetState
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum ResetState {
    Normal = 0,
    TriggerReset = 0xB6
}
