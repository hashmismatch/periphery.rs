use packed_struct::prelude::*;

#[derive(PackedStruct, Debug, Copy, Clone, Serialize)]
#[packed_struct(size_bytes="4")]
pub struct Crc {
    #[packed_field(endian="lsb")]
    pub crc: u32
}


#[derive(PackedStruct, Debug, Copy, Clone, Serialize)]
#[packed_struct(size_bytes="2", bit_numbering="lsb0")]
pub struct MessageHeader {
    #[packed_field(bits="14..12")]
    pub number_of_data_objects: Integer<u8, packed_bits::Bits3>,
    #[packed_field(bits="11..9")]
    pub message_id: Integer<u8, packed_bits::Bits3>,
    #[packed_field(bits="8", ty="enum")]
    pub port_power_role: PortPowerRole,
    #[packed_field(bits="7..6", ty="enum")]
    pub specification_revision: SpecificationRevision,
    #[packed_field(bits="5", ty="enum")]
    pub port_data_role: PortDataRole,
    #[packed_field(bits="3..0")]
    pub message_type: Integer<u8, packed_bits::Bits4>,
}

impl MessageHeader {
    pub fn decode(&self) -> Result<DecodedMessageHeader, ()> {
        if *self.number_of_data_objects == 0 {
            let control_message_type = ControlMessageType::from_primitive(*self.message_type).unwrap();
            Ok(DecodedMessageHeader::Control(ControlMessageHeader {
                message_id: self.message_id,
                port_power_role: self.port_power_role,
                specification_revision: self.specification_revision,
                port_data_role: self.port_data_role,                
                message_type: control_message_type                
            }))
        } else {
            let data_message_type = DataMessageType::from_primitive(*self.message_type).unwrap();
            Ok(DecodedMessageHeader::Data(DataMessageHeader {
                number_of_data_objects: self.number_of_data_objects,
                message_id: self.message_id,
                port_power_role: self.port_power_role,
                specification_revision: self.specification_revision,
                port_data_role: self.port_data_role,
                message_type: data_message_type,
            }))
        }
    }
}

#[derive(PrimitiveEnum_u8, Debug, Copy, Clone, PartialEq, Serialize)]
pub enum SpecificationRevision {
    Revision1 = 0b00,
    Revision2 = 0b01
}

#[derive(PrimitiveEnum_u8, Debug, Copy, Clone, PartialEq, Serialize)]
pub enum PortDataRole {
    /// Upstream Facing Port
    /// 
    /// The Upstream Facing Port or UFP is equivalent in the USB topology to the USB B-Port. The UFP will also correspond to the USB Device but only if USB Communication is supported while acting as a UFP. Products which charge can be a UFP while not having USB Communication capability.
    Ufp = 0,
    /// Downstream Facing Port
    /// 
    /// The Downstream Facing Port or DFP is equivalent in the USB topology to the USB A-Port. The DFP will also correspond to the USB Host but only if USB Communication is supported while acting as a DFP. Products such as Wall Warts can be a DFP while not having USB Communication capability. The DFP also acts as the bus master when controlling alternate mode operation.
    Dfp = 1
}

#[derive(PrimitiveEnum_u8, Debug, Copy, Clone, PartialEq, Serialize)]
pub enum PortPowerRole {
    Sink = 0,
    Source = 1
}

#[derive(PrimitiveEnum_u8, Debug, Copy, Clone, PartialEq, Serialize)]
pub enum ControlMessageType {
    GoodCrc = 0b0001,
    GotoMin = 0b0010,
    Accept =  0b0011,
    Reject =  0b0100,
    Ping =    0b0101,
    PsRdy =   0b0110,
    GetSourceCap = 0b0111,
    GetSinkCap =   0b1000,
    DrSwap = 0b1001,
    PrSwap = 0b1010,
    VconnSwap = 0b1011,
    Wait = 0b1100,
    SoftReset = 0b1101
}

#[derive(PrimitiveEnum_u8, Debug, Copy, Clone, PartialEq, Serialize)]
pub enum DataMessageType {
    SourceCapabilities = 0b0001,
    Request = 0b0010,
    Bist = 0b0011,
    SinkCapabilities = 0b0100,
    VendorDefined = 0b1111
}

#[derive(Copy, Clone, Debug, Serialize)]
pub struct DataMessageHeader {
    pub number_of_data_objects: Integer<u8, packed_bits::Bits3>,
    pub message_id: Integer<u8, packed_bits::Bits3>,
    pub port_power_role: PortPowerRole,
    pub specification_revision: SpecificationRevision,
    pub port_data_role: PortDataRole,
    pub message_type: DataMessageType
}

impl DataMessageHeader {
    pub fn get_number_of_data_bytes(&self) -> usize {
        (*self.number_of_data_objects) as usize * 4
    }
}

#[derive(Copy, Clone, Debug, Serialize)]
pub struct ControlMessageHeader {
    pub message_id: Integer<u8, packed_bits::Bits3>,
    pub port_power_role: PortPowerRole,
    pub specification_revision: SpecificationRevision,
    pub port_data_role: PortDataRole,
    pub message_type: ControlMessageType    
}

#[derive(Copy, Clone, Debug, Serialize)]
pub enum DecodedMessageHeader {
    Data(DataMessageHeader),
    Control(ControlMessageHeader)
}

impl DecodedMessageHeader {
    pub fn is_data_header(&self) -> bool {
        if let &DecodedMessageHeader::Data(_) = self {
            true
        } else {
            false
        }
    }
}


#[derive(PrimitiveEnum_u8, Debug, Copy, Clone, PartialEq, Serialize)]
pub enum SupplyKind {
    FixedSupply = 0,
    Battery = 1,
    VariableSupply = 2,
    Reserved = 3
}

#[derive(PackedStruct, Debug, Copy, Clone, Serialize)]
#[packed_struct(size_bytes="4", bit_numbering="lsb0")]
pub struct PowerDataObject {
    #[packed_field(bits="31..30", ty="enum")]
    pub supply_kind: SupplyKind
}

#[derive(PackedStruct, Debug, Copy, Clone, Serialize)]
#[packed_struct(size_bytes="4", bit_numbering="lsb0", endian="msb")]
pub struct PowerDataObjectFixed {
    #[packed_field(bits="31..30", ty="enum")]
    pub supply: SupplyKind,
    #[packed_field(bits="29")]
    pub dual_role_power: bool,
    #[packed_field(bits="28")]
    pub usb_suspend_supported: bool,
    #[packed_field(bits="27")]
    pub unconstrained_power: bool,
    #[packed_field(bits="26")]
    pub usb_communications_capable: bool,
    #[packed_field(bits="25")]
    pub dual_role_data: bool,
    #[packed_field(bits="21..20")]
    pub peak_current: Integer<u8, packed_bits::Bits2>,
    #[packed_field(bits="19..10")]
    pub voltage: Integer<u16, packed_bits::Bits10>,
    #[packed_field(bits="9..0")]
    pub maximum_current: Integer<u16, packed_bits::Bits10>
}

impl PowerDataObjectFixed {
    pub fn get_voltage_milli_volts(&self) -> u32 {
        *self.voltage as u32 * 50
    }

    pub fn get_maximum_current_milli_amperes(&self) -> u32 {
        *self.maximum_current as u32 * 10
    }
}


#[derive(PackedStruct, Debug, Copy, Clone, Serialize)]
#[packed_struct(size_bytes="4", bit_numbering="lsb0", endian="msb")]
pub struct PowerDataObjectVariable {
    #[packed_field(bits="31..30", ty="enum")]
    pub supply: SupplyKind,
    #[packed_field(bits="29..20")]
    pub maximum_voltage: Integer<u16, packed_bits::Bits10>,
    #[packed_field(bits="19..10")]
    pub minimum_voltage: Integer<u16, packed_bits::Bits10>,
    #[packed_field(bits="9..0")]
    pub maximum_current: Integer<u16, packed_bits::Bits10>
}

impl PowerDataObjectVariable {
    pub fn get_maximum_voltage_milli_volts(&self) -> u32 {
        *self.maximum_voltage as u32 * 50
    }

    pub fn get_minimum_voltage_milli_volts(&self) -> u32 {
        *self.minimum_voltage as u32 * 50
    }

    pub fn get_maximum_current_milli_amperes(&self) -> u32 {
        *self.maximum_current as u32 * 10
    }
}

#[derive(PackedStruct, Debug, Copy, Clone, Serialize)]
#[packed_struct(size_bytes="4", bit_numbering="lsb0", endian="msb")]
pub struct PowerDataObjectBattery {
    #[packed_field(bits="31..30", ty="enum")]
    pub supply: SupplyKind,
    #[packed_field(bits="29..20")]
    pub maximum_voltage: Integer<u16, packed_bits::Bits10>,
    #[packed_field(bits="19..10")]
    pub minimum_voltage: Integer<u16, packed_bits::Bits10>,
    #[packed_field(bits="9..0")]
    pub maximum_allowable_power: Integer<u16, packed_bits::Bits10>
}

impl PowerDataObjectBattery {
    pub fn get_maximum_voltage_milli_volts(&self) -> u32 {
        *self.maximum_voltage as u32 * 50
    }

    pub fn get_minimum_voltage_milli_volts(&self) -> u32 {
        *self.minimum_voltage as u32 * 50
    }

    pub fn get_maximum_allowable_power_milli_watts(&self) -> u32 {
        *self.maximum_allowable_power as u32 * 250
    }
}




#[derive(PackedStruct, Debug, Copy, Clone, Serialize)]
#[packed_struct(size_bytes="4", bit_numbering="lsb0", endian="msb")]
pub struct SinkPowerDataObjectFixed {
    #[packed_field(bits="31..30", ty="enum")]
    pub supply: SupplyKind,
    #[packed_field(bits="29")]
    pub dual_role_power: bool,
    #[packed_field(bits="28")]
    pub higher_capability: bool,
    #[packed_field(bits="27")]
    pub unconstrained_power: bool,
    #[packed_field(bits="26")]
    pub usb_communications_capable: bool,
    #[packed_field(bits="25")]
    pub dual_role_data: bool,    
    #[packed_field(bits="19..10")]
    pub voltage: Integer<u16, packed_bits::Bits10>,
    #[packed_field(bits="9..0")]
    pub operational_current: Integer<u16, packed_bits::Bits10>
}

impl SinkPowerDataObjectFixed {
    pub fn get_voltage_milli_volts(&self) -> u32 {
        *self.voltage as u32 * 50
    }

    pub fn get_operational_current_milli_amperes(&self) -> u32 {
        *self.operational_current as u32 * 10
    }
}

#[derive(PackedStruct, Debug, Copy, Clone, Serialize)]
#[packed_struct(size_bytes="4", bit_numbering="lsb0", endian="msb")]
pub struct SinkPowerDataObjectVariable {
    #[packed_field(bits="31..30", ty="enum")]
    pub supply: SupplyKind,
    #[packed_field(bits="29..20")]
    pub maximum_voltage: Integer<u16, packed_bits::Bits10>,
    #[packed_field(bits="19..10")]
    pub minimum_voltage: Integer<u16, packed_bits::Bits10>,
    #[packed_field(bits="9..0")]
    pub operational_current: Integer<u16, packed_bits::Bits10>
}

impl SinkPowerDataObjectVariable {
    pub fn get_maximum_voltage_milli_volts(&self) -> u32 {
        *self.maximum_voltage as u32 * 50
    }

    pub fn get_minimum_voltage_milli_volts(&self) -> u32 {
        *self.minimum_voltage as u32 * 50
    }

    pub fn get_operational_current_milli_amperes(&self) -> u32 {
        *self.operational_current as u32 * 10
    }
}


#[derive(PackedStruct, Debug, Copy, Clone, Serialize)]
#[packed_struct(size_bytes="4", bit_numbering="lsb0", endian="msb")]
pub struct SinkPowerDataObjectBattery {
    #[packed_field(bits="31..30", ty="enum")]
    pub supply: SupplyKind,
    #[packed_field(bits="29..20")]
    pub maximum_voltage: Integer<u16, packed_bits::Bits10>,
    #[packed_field(bits="19..10")]
    pub minimum_voltage: Integer<u16, packed_bits::Bits10>,
    #[packed_field(bits="9..0")]
    pub operational_power: Integer<u16, packed_bits::Bits10>
}

impl SinkPowerDataObjectBattery {
    pub fn get_maximum_voltage_milli_volts(&self) -> u32 {
        *self.maximum_voltage as u32 * 50
    }

    pub fn get_minimum_voltage_milli_volts(&self) -> u32 {
        *self.minimum_voltage as u32 * 50
    }

    pub fn get_operational_power_milli_watts(&self) -> u32 {
        *self.operational_power as u32 * 250
    }
}


#[derive(PackedStruct, Debug, Copy, Clone, Serialize)]
#[packed_struct(size_bytes="4", bit_numbering="lsb0", endian="msb")]
pub struct FixedAndVariableRequest {
    #[packed_field(bits="30..28")]
    pub object_position: Integer<u8, packed_bits::Bits3>,
    #[packed_field(bits="27")]
    pub give_back: bool,
    #[packed_field(bits="26")]
    pub capability_mismatch: bool,
    #[packed_field(bits="25")]
    pub usb_communications_capable: bool,
    #[packed_field(bits="24")]
    pub no_usb_suspend: bool,
    #[packed_field(bits="19..10")]
    pub operating_current: Integer<u16, packed_bits::Bits10>,
    #[packed_field(bits="9..0")]
    pub maximum_operating_current: Integer<u16, packed_bits::Bits10>    
}


pub fn amps_to_usb_pd(amps: f32) -> Option<Integer<u16, packed_bits::Bits10>> {
    // todo: ceiling, limits
    Some(((amps * 100.0) as u16).into())
}
