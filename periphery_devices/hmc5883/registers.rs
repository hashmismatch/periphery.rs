use periphery_core::*;
use periphery_core::prelude::v1::*;

use packed_struct::prelude::*;


#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum SamplesAveraged {
    Samples1 = 0,
    Samples2 = 1,
    Samples4 = 2,
    Samples8 = 3
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum DataOutputRate {
    Output_0_75Hz = 0,
    Output_1_5Hz = 1,
    Output_3Hz = 2,
    Output_7_5Hz = 3,
    Output_15Hz = 4,
    Output_30Hz = 5,
    Output_70Hz = 6
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum MeasurementMode {
    Normal = 0,
    PositiveBias = 1,
    NegativeBias = 2
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(bit_numbering="msb0")]
pub struct ConfigurationRegisterA {
    #[packed_field(bits="1..2", ty="enum")]
	pub samples: SamplesAveraged, 
    #[packed_field(bits="3..5", ty="enum")]
	pub data_output_rate: DataOutputRate,
    #[packed_field(bits="6..7", ty="enum")]
	pub measurement_mode: MeasurementMode
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum Gain {
    Gain_1370 = 0,
    Gain_1090 = 1,
    Gain_820 = 2,
    Gain_660 = 3,
    Gain_440 = 4,
    Gain_390 = 5,
    Gain_330 = 6,
    Gain_230 = 7,
}

impl Gain {
    pub fn get_lsb_per_gauss(&self) -> u16 {
        match *self {
            Gain::Gain_1370 => 1370,
            Gain::Gain_1090 => 1090,
            Gain::Gain_820  => 820,
            Gain::Gain_660  => 660,
            Gain::Gain_440  => 440,
            Gain::Gain_390  => 390,
            Gain::Gain_330  => 330,
            Gain::Gain_230  => 230
        }
    }
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(bit_numbering="msb0")]
pub struct ConfigurationRegisterB {
    #[packed_field(bits="0..2", ty="enum")]
    pub gain: Gain
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum OperatingMode {
    Continous = 0,
    SingleMeasurement = 1,
    Idle1 = 2,
    Idle2 = 3
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(bit_numbering="msb0")]
pub struct ModeRegister {
    #[packed_field(bits="0")]
    pub high_speed_i2c_enabled: bool,
    #[packed_field(bits="6..7", ty="enum")]
    pub operating_mode: OperatingMode
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(bit_numbering="msb0")]
pub struct StatusRegister {
    #[packed_field(bits="6")]
    pub lock: bool,
    pub ready: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(endian="msb")]
pub struct MagData {
    pub x: i16,    
    pub z: i16,
    pub y: i16
}
