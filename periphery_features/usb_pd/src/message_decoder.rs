use structs::*;
use packed_struct::*;

#[derive(Copy, Clone, Debug, Serialize)]
pub enum DecodingError {
    InvalidDataLength,
    Unsupported,
    UnpackingError(PackingError)
}

impl From<PackingError> for DecodingError {
    fn from(v: PackingError) -> Self {
        DecodingError::UnpackingError(v)
    }
}

pub fn decode_message(header: DecodedMessageHeader, data_bytes: &[u8]) -> Result<PdMessage, DecodingError> {
    let data = match header {
        DecodedMessageHeader::Control(control) => {
            PdData::NoData
        },
        DecodedMessageHeader::Data(data) => {
            match data.message_type {
                DataMessageType::SourceCapabilities => {
                    let mut caps: PdSourceCapabilities = Default::default();

                    for object_data in data_bytes.chunks(4) {
                        let data_bytes: Vec<_> = object_data.iter().cloned().rev().collect();
                        let pdo = PowerDataObject::unpack_from_slice(&data_bytes)?;

                        let k = match pdo.supply_kind {
                            SupplyKind::FixedSupply => {
                                PowerDataKind::Fixed(PowerDataObjectFixed::unpack_from_slice(&data_bytes)?)
                            },
                            SupplyKind::Battery => {
                                PowerDataKind::Battery(PowerDataObjectBattery::unpack_from_slice(&data_bytes)?)
                            },
                            SupplyKind::VariableSupply => {
                                PowerDataKind::Variable(PowerDataObjectVariable::unpack_from_slice(&data_bytes)?)
                            },
                            SupplyKind::Reserved => {
                                return Err(DecodingError::Unsupported);
                            }
                        };

                        caps.power.push(k);
                    }
                    
                    PdData::SourceCapabilities(caps)
                },

                DataMessageType::Request => {
                    if data_bytes.len() != 4 { panic!("huh? more?"); }

                    let data_bytes: Vec<_> = data_bytes.iter().cloned().rev().collect();
                    PdData::Request(PdRequest::FixedAndVariable(FixedAndVariableRequest::unpack_from_slice(&data_bytes)?))
                }

                _ => {
                    return Err(DecodingError::Unsupported);
                }
            }
        }
    };

    let msg = PdMessage {
        header: header,
        data: data
    };

    Ok(msg)
}



#[derive(Debug, Clone, Serialize)]
pub struct PdMessage {
    pub header: DecodedMessageHeader,
    pub data: PdData
}

impl PdMessage {
    pub fn new_control(header: ControlMessageHeader) -> Self {
        PdMessage {
            header: DecodedMessageHeader::Control(header),
            data: PdData::NoData
        }
    }

    pub fn new_data(header: DataMessageHeader, data: PdData) -> Self {
        PdMessage {
            header: DecodedMessageHeader::Data(header),
            data: data
        }
    }    

    pub fn get_message_id(&self) -> u8 {
        match self.header {
            DecodedMessageHeader::Data(h) => *h.message_id,
            DecodedMessageHeader::Control(h) => *h.message_id
        }
    }

    pub fn pack(&self) -> Vec<u8> {
        match self.header {
            DecodedMessageHeader::Control(h) => { Self::pack_control(h).iter().cloned().collect() },
            DecodedMessageHeader::Data(h) => { Self::pack_data(h, &self.data) }
        }
    }

    pub fn pack_control(header: ControlMessageHeader) -> [u8; 2] {
        let msg_header = MessageHeader {
            number_of_data_objects: 0.into(),
            message_id: header.message_id,
            port_power_role: header.port_power_role,
            specification_revision: header.specification_revision,
            port_data_role: header.port_data_role,
            message_type: header.message_type.to_primitive().into()
        };

        let header_packed = msg_header.pack();
        
        [
            header_packed[1], header_packed[0]
        ]
    }

    pub fn pack_data(header: DataMessageHeader, data: &PdData) -> Vec<u8> {
        use packed_struct::*;

        let mut ret = vec![];

        let msg_header = MessageHeader {
            number_of_data_objects: header.number_of_data_objects,
            message_id: header.message_id,
            port_power_role: header.port_power_role,
            specification_revision: header.specification_revision,
            port_data_role: header.port_data_role,
            message_type: header.message_type.to_primitive().into()
        };

        let header_packed = msg_header.pack();
        ret.push(header_packed[1]);
        ret.push(header_packed[0]);

        let mut data_packed = vec![];
        match data {
            &PdData::NoData => { panic!("no data?"); },
            &PdData::SourceCapabilities(ref s) => {
                for &p in &s.power {
                    match p {
                        PowerDataKind::Fixed(p) => { data_packed.extend(p.pack().iter().rev()); },
                        PowerDataKind::Battery(p) => { data_packed.extend(p.pack().iter().rev()); },
                        PowerDataKind::Variable(p) => { data_packed.extend(p.pack().iter().rev()); }
                    }                    
                }
            },
            &PdData::Request(ref r) => {
                match r {
                    &PdRequest::FixedAndVariable(ref r) => { data_packed.extend(r.pack().iter().rev()); }
                }
            }
        };
        
        ret.append(&mut data_packed);
        
        ret
    }

    pub fn packed_add_crc(data: &[u8]) -> Vec<u8> {
        use crc::{crc32, Hasher32};
        
        let mut ret: Vec<_> = data.iter().cloned().collect();
        let crc = crc32::checksum_ieee(&data);        
        let crc = Crc { crc: crc }.pack();
        ret.extend(&crc);

        ret
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum PdData {
    NoData,
    SourceCapabilities(PdSourceCapabilities),
    Request(PdRequest)
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct PdSourceCapabilities {
    pub power: Vec<PowerDataKind>
}

#[derive(Debug, Copy, Clone, Serialize)]
pub enum PowerDataKind {
    Fixed(PowerDataObjectFixed),
    Battery(PowerDataObjectBattery),
    Variable(PowerDataObjectVariable)
}

#[derive(Debug, Clone, Serialize)]
pub enum PdRequest {
    FixedAndVariable(FixedAndVariableRequest)
}


#[test]
fn test_roundtrip() {
    use fifo_decoder::*;

    {
        let header = ControlMessageHeader {
            message_id: 7.into(),
            port_power_role: PortPowerRole::Sink,
            specification_revision: SpecificationRevision::Revision2,
            port_data_role: PortDataRole::Ufp,
            message_type: ControlMessageType::Ping
        };

        let packed = PdMessage::pack_control(header);        

        println!("packed: {:?}", packed);

        let packed = PdMessage::packed_add_crc(&packed);

        let msg = FifoDecoder::decode_single_message(&packed).unwrap();
        println!("unpacked: {:?}", msg);
    }

    {
        let header = DataMessageHeader {
            number_of_data_objects: 1.into(),
            message_id: 6.into(),
            port_power_role: PortPowerRole::Sink,
            specification_revision: SpecificationRevision::Revision1,
            port_data_role: PortDataRole::Dfp,
            message_type: DataMessageType::Request
        };

        let data = PdData::Request(
            PdRequest::FixedAndVariable(FixedAndVariableRequest {
                object_position: 1.into(),
                give_back: false,
                capability_mismatch: false,
                usb_communications_capable: false,
                no_usb_suspend: true,
                operating_current: 240.into(),
                maximum_operating_current: 300.into()
            })
        );

        let packed = PdMessage::pack_data(header, &data);

        println!("packed: {:?}", packed);
        let packed = PdMessage::packed_add_crc(&packed);

        let msg = FifoDecoder::decode_single_message(&packed).unwrap();
        println!("unpacked: {:?}", msg);
    }
}