use periphery_core::*;
use periphery_core::prelude::v1::*;

use packed_struct::prelude::*;

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="3", endian="msb")]
pub struct MeasurementValue {
    pub value: Integer<u32, packed_bits::Bits20>
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum Oversampling {
    MeasurementSkipped = 0,
    Times1 = 0b001,
    Times2 = 0b010,
    Times4 = 0b011,
    Times8 = 0b100,
    Times16 = 0b101
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum IirFilter {
    FilterOff = 0,
    Filter2 = 1,
    Filter4 = 2,
    Filter8 = 3,
    Filter16 = 4
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum PowerMode {
    SleepMode = 0,
    ForcedMode = 1,
    ForcedMode1 = 2,
    NormalMode = 3
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum StandbyTime {
    Standby0_5ms = 0,
    Standby62_5ms = 1,
    Standby125ms = 2,
    Standby250ms = 3,
    Standby500ms = 4,
    Standby1000ms = 5,
    Standby2000ms = 6,
    Standby4000ms = 7
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct StatusRegister {
    #[packed_field(bits="3")]
    pub measuring: bool,
    #[packed_field(bits="0")]
    pub im_update: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct ControlMeasurementRegister {
    #[packed_field(bits="7..5", ty="enum")]
    pub oversampling_temperature: Oversampling,
    #[packed_field(bits="4..2", ty="enum")]
    pub oversampling_pressure: Oversampling,
    #[packed_field(bits="1..0", ty="enum")]
    pub power_mode: PowerMode    
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct ConfigurationRegister {
    #[packed_field(bits="7..5", ty="enum")]
    pub standby_time: StandbyTime,
    #[packed_field(bits="4..2", ty="enum")]
    pub iir_filter: IirFilter,
    #[packed_field(bits="0")]
    pub enable_3wire_spi: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct ResetRegister {
    #[packed_field(bits="7..0", ty="enum")]
    pub state: ResetState
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum ResetState {
    Normal = 0,
    TriggerReset = 0xB6
}





#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(endian="lsb")]
pub struct CalibrationCoefficients {
    pub t1: u16,
    pub t2: i16,
    pub t3: i16,
    pub p1: u16,
    pub p2: i16,
    pub p3: i16,
    pub p4: i16,
    pub p5: i16,
    pub p6: i16,
    pub p7: i16,
    pub p8: i16,
    pub p9: i16
}

