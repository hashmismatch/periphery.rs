use periphery_core::*;
use periphery_core::prelude::v1::*;

use packed_struct::*;


#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum ExtSync {
    InputDisabled = 0,
    Temperature = 1,
    GyroX = 2,
    GyroY = 3,
    GyroZ = 4,
    AccelX = 5,
    AccelY = 6,
    AccelZ = 7
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum DigitalLowPassFilter {
    /// Filter is disabled
    Filter0 = 0,
    Filter1 = 1,
    Filter2 = 2,
    Filter3 = 3,
    Filter4 = 4,
    Filter5 = 5,
    Filter6 = 6,
    /// Filter is disabled
    FilterReserved = 7
}

impl DigitalLowPassFilter {
    pub fn is_disabled(&self) -> bool {
        *self == DigitalLowPassFilter::Filter0 ||
        *self == DigitalLowPassFilter::FilterReserved
    }

    pub fn get_gyroscope_output_rate_hz(&self) -> u32 {
        if self.is_disabled() {
            8000
        } else {
            1000
        }
    } 
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering="msb0")]
pub struct ConfigRegister {
    #[packed_field(bits="2..4", ty="enum")]
	pub ext_sync_set: ExtSync,
    #[packed_field(bits="5..7", ty="enum")]
	pub dlpf_cfg: DigitalLowPassFilter
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
/// +/- Degrees per second
pub enum GyroFullScale {    	
    Scale_250 = 0,
    Scale_500 = 1,
    Scale_1000 = 2,
    Scale_2000 = 3
}

impl GyroFullScale {
    pub fn get_lsb_per_deg_per_s(&self) -> f32 {
        match *self {
            GyroFullScale::Scale_250 => 131.0,
            GyroFullScale::Scale_500 => 65.5,
            GyroFullScale::Scale_1000 => 32.8,
            GyroFullScale::Scale_2000 => 16.4
        }
    }
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering="msb0")]
pub struct GyroConfig {
    #[packed_field(bits="0")]
	pub x_axis_self_test_enabled: bool,
    #[packed_field(bits="1")]
	pub y_axis_self_test_enabled: bool,
    #[packed_field(bits="2")]
	pub z_axis_self_test_enabled: bool,
    #[packed_field(bits="3..4", ty="enum")]
	pub scale: GyroFullScale
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum AccelerometerFullScale {    	
    Scale_2g = 0,
    Scale_4g = 1,
    Scale_8g = 2,
    Scale_16g = 3
}


impl AccelerometerFullScale {
    pub fn get_lsb_per_g(&self) -> i16 {
        match *self {
            AccelerometerFullScale::Scale_2g => 16384,
            AccelerometerFullScale::Scale_4g => 8192,
            AccelerometerFullScale::Scale_8g => 4096,
            AccelerometerFullScale::Scale_16g => 2048
        }
    }
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering="msb0")]
pub struct AccelConfig {
    #[packed_field(bits="0")]
	pub x_axis_self_test_enabled: bool,
    #[packed_field(bits="1")]
	pub y_axis_self_test_enabled: bool,
    #[packed_field(bits="2")]
	pub z_axis_self_test_enabled: bool,
    #[packed_field(bits="3..4", ty="enum")]
	pub scale: AccelerometerFullScale
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum ClockSource {
    Internal = 0,
    PllGyroX = 1,
    PllGyroY = 2,
    PllGyroZ = 3,
    PllExternal32KHz = 4,
    PllExternal19MHz = 5,
    Reserved = 6,
    Stop = 7
}


#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering="msb0")]
pub struct PowerManagement1 {
    #[packed_field(bits="0")]
	pub device_reset: bool,
    #[packed_field(bits="1")]
	pub sleep: bool,
    #[packed_field(bits="2")]
	pub cycle: bool,
    #[packed_field(bits="4")]
	pub temperature_disabled: bool,
    #[packed_field(bits="5..7", ty="enum")]
	pub clock_source: ClockSource
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering="msb0")]
pub struct SignalPathReset {
    #[packed_field(bits="5")]
	pub gyro_reset: bool,
    #[packed_field(bits="6")]
	pub accel_reset: bool,
    #[packed_field(bits="7")]
	pub temperature_reset: bool
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering="msb0")]
pub struct InterruptPinConfig {
    #[packed_field(bits="0")]
	pub int_level: bool,
    #[packed_field(bits="1")]
	pub int_open: bool,
    #[packed_field(bits="2")]
	pub latch_interrupt_enable: bool,
    #[packed_field(bits="3")]
	pub interrupt_ready_clear: bool,
    #[packed_field(bits="4")]
	pub fsync_int_level: bool,
    // todo: check specs?
    #[packed_field(bits="6")]
	pub fsync_int_en: bool
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering="msb0")]
pub struct InterruptEnable {
    #[packed_field(bits="3")]
	pub fifo_oflow_en: bool,
    #[packed_field(bits="4")]
	pub i2c_mst_int_en: bool,
    #[packed_field(bits="7")]
	pub data_rdy_en: bool
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering="msb0")]
pub struct InterruptStatus {
    #[packed_field(bits="3")]
	pub fifo_oflow_int: bool,
    #[packed_field(bits="4")]
	pub i2c_mst_int: bool,
    #[packed_field(bits="7")]
	pub data_rdy_int: bool
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering="msb0")]
pub struct UserControl {
    #[packed_field(bits="1")]
	pub fifo_en: bool,
    #[packed_field(bits="2")]
	pub i2c_mst_en: bool,
    #[packed_field(bits="3")]
	pub i2c_if_dis: bool,
    #[packed_field(bits="5")]
	pub fifo_reset: bool,
    #[packed_field(bits="6")]
	pub i2c_mst_reset: bool,
    #[packed_field(bits="7")]
	pub sig_cond_reset: bool
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum WakeCtrl {
    Freq_1_25Hz = 0,
    Freq_5Hz = 1,
    Freq_20Hz = 2,
    Freq_40Hz = 3
}

#[derive(PackedStruct, Copy, Clone, Debug, PartialEq)]
#[packed_struct(bit_numbering="msb0")]
pub struct PowerManagement2 {
    #[packed_field(bits="0..1", ty="enum")]
	pub lp_wake_ctrl: WakeCtrl,
    #[packed_field(bits="2")]
	pub stby_xa: bool,
    #[packed_field(bits="3")]
	pub stby_ya: bool,
    #[packed_field(bits="4")]
	pub stby_za: bool,
    #[packed_field(bits="5")]
	pub stby_xg: bool,
    #[packed_field(bits="6")]
	pub stby_yg: bool,
    #[packed_field(bits="7")]
	pub stby_zg: bool
}




#[derive(Debug, Copy, Clone, PackedStruct)]
#[packed_struct(endian="msb")]
pub struct AcceleratorData {
    pub x: i16,
    pub y: i16,
    pub z: i16
}

#[derive(Debug, Copy, Clone, PackedStruct)]
#[packed_struct(endian="msb")]
pub struct TemperatureData {
    pub temperature: i16,
}

#[derive(Debug, Copy, Clone, PackedStruct)]
#[packed_struct(endian="msb")]
pub struct GyroscopeData {
    pub x: i16,
    pub y: i16,
    pub z: i16
}

#[derive(Debug, Copy, Clone, PackedStruct)]
pub struct AccTempGyroData {
    #[packed_field(size_bytes="6")]
    pub acceleration: AcceleratorData,
    #[packed_field(size_bytes="2")]
    pub temperature: TemperatureData,
    #[packed_field(size_bytes="6")]
    pub gyroscope: GyroscopeData
}
