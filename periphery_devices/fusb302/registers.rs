use periphery_core::*;
use periphery_core::prelude::v1::*;

use packed_struct::prelude::*;

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct DeviceId {
    #[packed_field(bits="7..4")]
    pub version: Integer<u8, packed_bits::Bits4>,
    #[packed_field(bits="3..0")]
    pub revision: Integer<u8, packed_bits::Bits4>
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Switches0 {
    #[packed_field(bits="7")]
    pub pu_en2: bool,
    #[packed_field(bits="6")]
    pub pu_en1: bool,
    #[packed_field(bits="5")]
    pub vcon_cc2: bool,
    #[packed_field(bits="4")]
    pub vcon_cc1: bool,
    #[packed_field(bits="3")]
    pub meas_cc2: bool,
    #[packed_field(bits="2")]
    pub meas_cc1: bool,
    #[packed_field(bits="1")]
    pub pdwn2: bool,
    #[packed_field(bits="0")]
    pub pdwn1: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Switches1 {
    #[packed_field(bits="7")]
    pub powerrole: bool,
    #[packed_field(bits="6")]
    pub specrev1: bool,
    #[packed_field(bits="5")]
    pub sprecrev0: bool,
    #[packed_field(bits="4")]
    pub datarole: bool,
    #[packed_field(bits="2")]
    pub auto_crc: bool,
    #[packed_field(bits="1")]
    pub txcc2: bool,
    #[packed_field(bits="0")]
    pub txcc1: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Measure {
    #[packed_field(bits="6")]
    pub meas_vbus: bool,
    #[packed_field(bits="5..0")]
    pub mdac: Integer<u8, packed_bits::Bits6>,
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Slice {
    #[packed_field(bits="7..6")]
    pub sdac_hys: Integer<u8, packed_bits::Bits2>,
    #[packed_field(bits="5..0")]
    pub sdac: Integer<u8, packed_bits::Bits6>,
}


#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Control0 {
    #[packed_field(bits="6")]
    pub tx_flush: bool,
    #[packed_field(bits="5")]
    pub int_mask: bool,
    #[packed_field(bits="3..2")]
    pub host_cur: Integer<u8, packed_bits::Bits2>,
    #[packed_field(bits="1")]
    pub auto_pre: bool,
    #[packed_field(bits="0")]
    pub tx_start: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Control1 {
    #[packed_field(bits="6")]
    pub ensop2db: bool,
    #[packed_field(bits="5")]
    pub ensop1db: bool,
    #[packed_field(bits="4")]
    pub bist_mode2: bool,
    #[packed_field(bits="2")]
    pub rx_flush: bool,
    #[packed_field(bits="1")]
    pub ensop2: bool,
    #[packed_field(bits="0")]
    pub ensop1: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Control2 {
    #[packed_field(bits="7..6")]
    pub tog_save_pwr: Integer<u8, packed_bits::Bits2>,
    #[packed_field(bits="5")]
    pub tog_rd_only: bool,    
    #[packed_field(bits="3")]
    pub wake_en: bool,
    #[packed_field(bits="2..1")]
    pub mode: Integer<u8, packed_bits::Bits2>,
    #[packed_field(bits="0")]
    pub toggle: bool
}

#[derive(PackedStruct, Debug, Default, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Control3 {
    #[packed_field(bits="6")]
    pub send_hard_reset: bool,
    #[packed_field(bits="4")]
    pub auto_hardreset: bool,    
    #[packed_field(bits="3")]
    pub auto_softreset: bool,
    #[packed_field(bits="2..1")]
    pub n_retries: Integer<u8, packed_bits::Bits2>,
    #[packed_field(bits="0")]
    pub auto_retry: bool
}


#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Mask {
    #[packed_field(bits="7")]
    pub m_vbusok: bool,
    #[packed_field(bits="6")]
    pub m_activity: bool,
    #[packed_field(bits="5")]
    pub m_comp_chng: bool,
    #[packed_field(bits="4")]
    pub m_crc_chk: bool,
    #[packed_field(bits="3")]
    pub m_alert: bool,
    #[packed_field(bits="2")]
    pub m_wake: bool,
    #[packed_field(bits="1")]
    pub m_collision: bool,
    #[packed_field(bits="0")]
    pub m_bc_lvl: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Power {    
    #[packed_field(bits="3..0")]
    pub pwr: Integer<u8, packed_bits::Bits4>
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Reset {    
    #[packed_field(bits="1")]
    pub pd_reset: bool,
    #[packed_field(bits="0")]
    pub sw_reset: bool
}



#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct OcpReg {    
    #[packed_field(bits="3")]
    pub ocp_range: bool,
    #[packed_field(bits="2..0")]
    pub ocp_cur: Integer<u8, packed_bits::Bits3>
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct MaskA {    
    #[packed_field(bits="7")]
    pub m_ocp_temp: bool,
    #[packed_field(bits="6")]
    pub m_togdone: bool,
    #[packed_field(bits="5")]
    pub m_softfail: bool,
    #[packed_field(bits="4")]
    pub m_retryfail: bool,
    #[packed_field(bits="3")]
    pub m_hardsent: bool,
    #[packed_field(bits="2")]
    pub m_txsent: bool,
    #[packed_field(bits="1")]
    pub m_softrst: bool,
    #[packed_field(bits="0")]
    pub m_hardrst: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct MaskB {    
    #[packed_field(bits="0")]
    pub m_gcrcsent: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Control4 {    
    #[packed_field(bits="0")]
    pub tog_usrc_exit: bool
}


#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Status0A {    
    #[packed_field(bits="5")]
    pub softfail: bool,
    #[packed_field(bits="4")]
    pub retryfail: bool,
    #[packed_field(bits="3")]
    pub power3: bool,
    #[packed_field(bits="2")]
    pub power2: bool,
    #[packed_field(bits="1")]
    pub softrst: bool,
    #[packed_field(bits="0")]
    pub hardrst: bool
}


#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Status1A {    
    #[packed_field(bits="5..3")]
    pub togss: Integer<u8, packed_bits::Bits3>,
    #[packed_field(bits="2")]
    pub rxsop2db: bool,
    #[packed_field(bits="1")]
    pub rxsop1db: bool,
    #[packed_field(bits="0")]
    pub rxsop: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct InterruptA {    
    #[packed_field(bits="7")]
    pub i_ocp_temp: bool,
    #[packed_field(bits="6")]
    pub i_togdone: bool,
    #[packed_field(bits="5")]
    pub i_softfail: bool,
    #[packed_field(bits="4")]
    pub i_retryfail: bool,
    #[packed_field(bits="3")]
    pub i_hardsent: bool,
    #[packed_field(bits="2")]
    pub i_txsent: bool,
    #[packed_field(bits="1")]
    pub i_softrst: bool,
    #[packed_field(bits="0")]
    pub i_hardrst: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct InterruptB {    
    #[packed_field(bits="0")]
    pub i_gcrcsent: bool
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Status0 {    
    #[packed_field(bits="7")]
    pub vbusok: bool,
    #[packed_field(bits="6")]
    pub activity: bool,
    #[packed_field(bits="5")]
    pub comp: bool,
    #[packed_field(bits="4")]
    pub crc_chk: bool,
    #[packed_field(bits="3")]
    pub alert: bool,
    #[packed_field(bits="2")]
    pub wake: bool,
    #[packed_field(bits="1..0")]
    pub bc_lvl: Integer<u8, packed_bits::Bits2>
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Status1 {    
    #[packed_field(bits="7")]
    pub rxsop2: bool,
    #[packed_field(bits="6")]
    pub rxsop1: bool,
    #[packed_field(bits="5")]
    pub rx_empty: bool,
    #[packed_field(bits="4")]
    pub rx_full: bool,
    #[packed_field(bits="3")]
    pub tx_empty: bool,
    #[packed_field(bits="2")]
    pub tx_full: bool,
    #[packed_field(bits="1")]
    pub ovrtemp: bool,
    #[packed_field(bits="0")]
    pub ocp: bool
}


#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(size_bytes="1", bit_numbering="lsb0")]
pub struct Interrupt {    
    #[packed_field(bits="7")]
    pub i_vbusok: bool,
    #[packed_field(bits="6")]
    pub i_activity: bool,
    #[packed_field(bits="5")]
    pub i_comp_chng: bool,
    #[packed_field(bits="4")]
    pub i_crc_chk: bool,
    #[packed_field(bits="3")]
    pub i_alert: bool,
    #[packed_field(bits="2")]
    pub i_wake: bool,
    #[packed_field(bits="1")]
    pub i_collision: bool,
    #[packed_field(bits="0")]
    pub i_bc_lvl: bool
}

