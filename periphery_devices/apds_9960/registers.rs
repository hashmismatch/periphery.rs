use periphery_core::*;
use periphery_core::prelude::v1::*;

use packed_struct::*;


#[derive(Copy, Clone, Debug, PartialEq, PrimitiveEnum_u8)]
pub enum ChipIdKind {
    Apds9960_Id_1 = 0xAB,
    Apds9960_Id_2 = 0x9C, 
    /// ??? my chip contains this value, never seen before?
    Apds9960_Id_3 = 0xA8,
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(bit_numbering="msb0")]
pub struct ChipId {
    #[packed_field(bits="0..7", ty="enum")]
    pub id: ChipIdKind
}


#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct Status {
    pub clear_photodiode_saturation: bool,
    pub pg_saturation: bool,
    pub proximity_interrupt: bool,
    pub als_interrupt: bool,
    // reserved bit
    #[packed_field(bits="5")]
    pub gesture_interrupt: bool,
    pub proximity_valid: bool,
    pub als_valid: bool
}


#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="8", bit_numbering="msb0", endian="lsb")]
pub struct RGBCData {
    pub clear: u16,
    pub red: u16,
    pub green: u16,
    pub blue: u16
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct EnableRegister {    
    #[packed_field(bits="1")]
    pub gesture_enable: bool,
    pub proximity_interrupt_enable: bool,
    pub als_interrupt_enable: bool,
    pub wait_enable: bool,
    pub proximity_detect_enable: bool,
    pub als_enable: bool,
    pub power_on: bool
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(endian="lsb")]
pub struct AlsInterruptThreshold {   
    pub low_interrupt_threshold: u16,
    pub high_interrupt_threshold: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct Persistence {
    #[packed_field(bits="0..3")]
    pub proximity_interrupt_persistence: UIntBits4,
    #[packed_field(bits="4..7")]
    pub als_interrupt_persistence: UIntBits4
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct Config1 {
    #[packed_field(bits="6")]
    pub wait_long: bool
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct ProximityPulseCount {
    #[packed_field(bits="0..1")]
    pub proximity_pulse_length: UIntBits2,
    #[packed_field(bits="2..7")]
    pub proximity_pulse_count: UIntBits6,
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct ControlRegister1 {
    #[packed_field(bits="0..1")]
    pub led_drive_strength: UIntBits2,
    #[packed_field(bits="4..5", ty="enum")]
    pub proximity_gain_control: ProximityGain,
    #[packed_field(bits="6..7", ty="enum")]
    pub als_and_color_gain: AlsGain
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct Config2 {
    pub proximity_saturation_interrupt_enable: bool,
    pub clear_photodiode_interrupt_enable: bool,
    #[packed_field(bits="3..4")]
    pub led_boost: UIntBits2,
    #[packed_field(bits="7")]
    pub reserved_true: bool
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct Config3 {
    #[packed_field(bits="2")]
    pub proximity_gain_compensation_enable: bool,
    pub sleep_after_interrupt: bool,
    pub proximity_mask_up: bool,
    pub proximity_mask_down: bool,
    pub proximity_mask_left: bool,
    pub proximity_mask_right: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
pub struct AdcIntegrationTime {
    pub value: u8
}

impl AdcIntegrationTime {
    pub fn new_from_cycles(cycles: u8) -> Self {
        let cycles = max(1, cycles);
        let value = (256 as u16 - cycles as u16) as u8;
        
        AdcIntegrationTime {
            value: value
        }
    }

    pub fn get_number_of_integration_cycles(&self) -> u16 {
        256 - (self.value as u16)
    }

    pub fn get_integration_time_ms(&self) -> f32 {
        self.get_number_of_integration_cycles() as f32 * 2.78
    }

    pub fn get_max_value(&self) -> u16 {
        let max_value = self.get_number_of_integration_cycles() as u32 * 1025;
        min(65535, max_value) as u16
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct GestureConfig1 {
    #[packed_field(bits="0..1")]
    pub gesture_fifo_threshold: UIntBits2,
    #[packed_field(bits="2..5")]
    pub gesture_exit_mask: UIntBits4,
    #[packed_field(bits="6..7")]
    pub gesture_exit_persistence: UIntBits2
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct GestureConfig2 {
    #[packed_field(bits="1..2", ty="enum")]
    pub gesture_gain: GestureGain,
    #[packed_field(bits="3..4", ty="enum")]
    pub gesture_led_drive_strength: GestureLedDrive,
    #[packed_field(bits="5..7")]
    pub gesture_wait_time: UIntBits3
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct GesturePulseCount {
    #[packed_field(bits="0..1")]
    pub gesture_pulse_length: UIntBits2,
    #[packed_field(bits="2..7")]
    pub number_of_gesture_pulses: UIntBits6
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct GestureConfig3 {
    #[packed_field(bits="6..7")]
    pub gesture_dimension: UIntBits2
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct GestureConfig4 {
    #[packed_field(bits="5")]
    pub gesture_fifo_clear: bool,
    #[packed_field(bits="6")]
    pub gesture_interrupt_enable: bool,
    #[packed_field(bits="7")]
    pub gesture_mode: bool
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
#[packed_struct(size_bytes="1", bit_numbering="msb0")]
pub struct GestureStatus {
    #[packed_field(bits="6")]
    pub gesture_fifo_overflow: bool,
    pub gesture_fifo_data_valid: bool
}

#[derive(Copy, Clone, Debug, PartialEq, PackedStruct)]
pub struct GestureFifo {
    pub up: u8,
    pub down: u8,
    pub left: u8,
    pub right: u8
}

impl GestureFifo {
    pub fn get_all_axes(&self) -> [u8; 4] {
        [self.up, self.down, self.left, self.right]
    }
}


#[derive(Copy, Clone, Debug, PartialEq, PrimitiveEnum_u8)]
pub enum ProximityGain {
    Times1 = 0,
    Times2 = 1,
    Times4 = 2,
    Times8 = 3
}

#[derive(Copy, Clone, Debug, PartialEq, PrimitiveEnum_u8)]
pub enum AlsGain {
    Times1 = 0,
    Times4 = 1,
    Times16 = 2,
    Times64 = 3
}

impl AlsGain {
    pub fn get_gain_factor(&self) -> u16 {
        match *self {
            AlsGain::Times1 => 1,
            AlsGain::Times4 => 4,
            AlsGain::Times16 => 16,
            AlsGain::Times64 => 64
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PrimitiveEnum_u8)]
pub enum GestureGain {
    Times1 = 0,
    Times2 = 1,
    Times4 = 2,
    Times8 = 3
}

impl GestureGain {
    pub fn get_gain_multiplier(&self) -> f32 {
        match *self {
            GestureGain::Times1 => 1.0,
            GestureGain::Times2 => 2.0,
            GestureGain::Times4 => 4.0,
            GestureGain::Times8 => 8.0
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PrimitiveEnum_u8)]
pub enum GestureLedDrive {
    Current100mA = 0,
    Current50mA = 1,
    Current25mA = 2,
    Current12_5mA = 3
}
