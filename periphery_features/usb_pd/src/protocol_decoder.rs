use fsm::*;

use packed_struct::prelude::*;
use message_decoder::*;
use structs::*;

#[derive(Default, Debug, Serialize)]
pub struct PdProtocolContext {
    raw_packet: Vec<u8>,
    data: Vec<u8>,
    header: Option<DecodedMessageHeader>
}

#[derive(Copy, Clone, Debug, Serialize)]
pub enum ProtocolErrorKind {
    DataHeaderObjectCount,
    InvalidHeader,
    CrcMismatch,
    MessageDecodingError(DecodingError),
    UnknownError
}


// events
#[derive(Copy, Clone, Debug, Serialize)]
pub struct ErrorOccurred(ProtocolErrorKind);
impl FsmEvent for ErrorOccurred {}

#[derive(Clone, Debug, Serialize)]
pub struct ToMessageDecoded(PdMessage);
impl FsmEvent for ToMessageDecoded {}

#[derive(Copy, Clone, Debug, Serialize)]
pub struct ByteReceived {
    pub byte: u8
}
impl FsmEvent for ByteReceived {}
impl ByteReceived {
    fn on_byte_received(&self, event_context: &mut EventContext<PdProtocol>) {
        event_context.context.raw_packet.push(self.byte);
    }
}

pub struct OnByteReceived;

pub struct HeaderDataRequired;
impl FsmGuard<PdProtocol, ByteReceived> for HeaderDataRequired {
    fn guard(event: &ByteReceived, event_context: &EventContext<PdProtocol>, states: &PdProtocolStatesStore) -> bool {
        let header: &ReceiveHeader = states.get_state();
        header.bytes_required > 0
    }
}


impl FsmActionSelf<PdProtocol, ReceiveHeader, ByteReceived> for OnByteReceived {
    fn action(event: &ByteReceived, event_context: &mut EventContext<PdProtocol>, state: &mut ReceiveHeader) {
        event.on_byte_received(event_context);

        // warning: the bytes are intentionally swapped!
        state.bytes_required -= 1;
        state.data[state.bytes_required] = event.byte;

        if state.bytes_required == 0 {
            if let Ok(h) = MessageHeader::unpack(&state.data) {
                if let Ok(h) = h.decode() {
                    event_context.context.header = Some(h);                    
                    return;
                }
            }
            
            event_context.queue.enqueue_event(ErrorOccurred(ProtocolErrorKind::InvalidHeader).into());
        }
    }
}

pub struct SomeDataRequired;
impl FsmGuard<PdProtocol, ByteReceived> for SomeDataRequired {
    fn guard(event: &ByteReceived, event_context: &EventContext<PdProtocol>, states: &PdProtocolStatesStore) -> bool {
        if let Some(ref header) = event_context.context.header {
            header.is_data_header()
        } else {
            false
        }
    }
}

pub struct SomeDataNotRequired;
impl FsmGuard<PdProtocol, ByteReceived> for SomeDataNotRequired {
    fn guard(event: &ByteReceived, event_context: &EventContext<PdProtocol>, states: &PdProtocolStatesStore) -> bool {
        if let Some(ref header) = event_context.context.header {
            !header.is_data_header()
        } else {
            false
        }
    }
}

pub struct MoreDataRequired;
impl FsmGuard<PdProtocol, ByteReceived> for MoreDataRequired {
    fn guard(event: &ByteReceived, event_context: &EventContext<PdProtocol>, states: &PdProtocolStatesStore) -> bool {
        let state: &ReceiveData = states.get_state();
        state.bytes_required > 0
    }
}


impl FsmActionSelf<PdProtocol, ReceiveData, ByteReceived> for OnByteReceived {
    fn action(event: &ByteReceived, event_context: &mut EventContext<PdProtocol>, state: &mut ReceiveData) {
        event.on_byte_received(event_context);
    
        state.bytes_required -= 1;
        state.data.push(event.byte);
    }
}

impl FsmActionSelf<PdProtocol, ReceiveCrc, ByteReceived> for OnByteReceived {
    fn action(event: &ByteReceived, event_context: &mut EventContext<PdProtocol>, state: &mut ReceiveCrc) {
        event.on_byte_received(event_context);
    
        state.bytes_required -= 1;
        state.data[state.bytes_required] = event.byte;
    }
}

impl FsmAction<PdProtocol, ReceiveHeader, ByteReceived, ReceiveData> for OnByteReceived {
    fn action(event: &ByteReceived, event_context: &mut EventContext<PdProtocol>, source_state: &mut ReceiveHeader, target_state: &mut ReceiveData) {
        if let Some(h) = event_context.context.header {
            if let DecodedMessageHeader::Data(data_header) = h {
                
                let data_bytes = data_header.get_number_of_data_bytes();
                target_state.bytes_required = data_bytes;

                <OnByteReceived as FsmActionSelf<PdProtocol, ReceiveData, ByteReceived>>::action(event, event_context, target_state);                
                return;
            }
        }

        event_context.queue.enqueue_event(ErrorOccurred(ProtocolErrorKind::UnknownError).into());
    }
}

impl FsmAction<PdProtocol, ReceiveHeader, ByteReceived, ReceiveCrc> for OnByteReceived {
    fn action(event: &ByteReceived, event_context: &mut EventContext<PdProtocol>, source_state: &mut ReceiveHeader, target_state: &mut ReceiveCrc) {
        target_state.bytes_required = 4;
        <OnByteReceived as FsmActionSelf<PdProtocol, ReceiveCrc, ByteReceived>>::action(event, event_context, target_state);
    }
}

impl FsmAction<PdProtocol, ReceiveData, ByteReceived, ReceiveCrc> for OnByteReceived {
    fn action(event: &ByteReceived, event_context: &mut EventContext<PdProtocol>, source_state: &mut ReceiveData, target_state: &mut ReceiveCrc) {
        target_state.bytes_required = 4;
        <OnByteReceived as FsmActionSelf<PdProtocol, ReceiveCrc, ByteReceived>>::action(event, event_context, target_state);
    }
}

pub struct MoreCrcRequired;
impl FsmGuard<PdProtocol, ByteReceived> for MoreCrcRequired {
    fn guard(event: &ByteReceived, event_context: &EventContext<PdProtocol>, states: &PdProtocolStatesStore) -> bool {
        let state: &ReceiveCrc = states.get_state();
        state.bytes_required > 1
    }
}

impl FsmAction<PdProtocol, ReceivingData, ToMessageDecoded, MessageDecoded> for ToMessageDecoded {
    fn action(event: &ToMessageDecoded, event_context: &mut EventContext<PdProtocol>, source_state: &mut ReceivingData, target_state: &mut MessageDecoded) {
        target_state.message = Some(event.0.clone());
    }
}

impl FsmAction<PdProtocol, ReceiveCrc, ByteReceived, MessageReceived> for OnByteReceived {
    fn action(event: &ByteReceived, event_context: &mut EventContext<PdProtocol>, source_state: &mut ReceiveCrc, target_state: &mut MessageReceived) {
        <OnByteReceived as FsmActionSelf<PdProtocol, ReceiveCrc, ByteReceived>>::action(event, event_context, source_state);

        // decode the message
        {
            // note: the bytes were already swapped when being written into the array
            let received_crc: u32 = **<MsbInteger<_, _, Integer<u32, packed_bits::Bits32>>>::unpack_from_slice(&source_state.data).unwrap();
            let computed_crc = {
                use crc::{crc32, Hasher32};

                let l = event_context.context.raw_packet.len();
                let crc_payload = &event_context.context.raw_packet[0..(l-4)];
                crc32::checksum_ieee(crc_payload)
            };

            if received_crc != computed_crc {
                event_context.queue.enqueue_event(ErrorOccurred(ProtocolErrorKind::CrcMismatch).into());
                return;
            }

            if let Some(header) = event_context.context.header {
                match decode_message(header, &event_context.context.data) {
                    Ok(msg) => {
                        event_context.queue.enqueue_event(ToMessageDecoded(msg).into());
                    },
                    Err(e) => {
                        event_context.queue.enqueue_event(ErrorOccurred(ProtocolErrorKind::MessageDecodingError(e)).into());
                    }
                }
            } else {
                event_context.queue.enqueue_event(ErrorOccurred(ProtocolErrorKind::UnknownError).into());
            }
        }
    }
}



// states
#[derive(Clone, Debug, Default, Serialize)]
pub struct ReceiveHeader {
    bytes_required: usize,
    data: [u8; 2]
}
impl FsmState<PdProtocol> for ReceiveHeader { 
    fn on_entry(&mut self, event_context: &mut EventContext<PdProtocol>) {
        event_context.context.raw_packet.clear();
        self.bytes_required = 2;
    }
}

#[derive(Clone, PartialEq, Default, Debug, Serialize)]
pub struct ReceiveData {
    bytes_required: usize,
    data: Vec<u8>
}
impl FsmState<PdProtocol> for ReceiveData {
    fn on_exit(&mut self, event_context: &mut EventContext<PdProtocol>) {
        event_context.context.data = self.data.clone();
    }
}

#[derive(Clone, PartialEq, Debug, Default, Serialize)]
pub struct ReceiveCrc {
    bytes_required: usize,
    data: [u8; 4]
}
impl FsmState<PdProtocol> for ReceiveCrc { }

#[derive(Default, Debug, Serialize)]
pub struct ReceivingData;
impl FsmState<PdProtocol> for ReceivingData { }

#[derive(Default, Debug, Serialize)]
pub struct MessageDecoded {
    pub message: Option<PdMessage>
}
impl FsmState<PdProtocol> for MessageDecoded { }

#[derive(Default, Debug, Serialize)]
pub struct MessageReceived;
impl FsmState<PdProtocol> for MessageReceived { }

#[derive(Default, Debug, Serialize)]
pub struct ProtocolError {
    pub error: Option<ProtocolErrorKind>
}
impl FsmState<PdProtocol> for ProtocolError { }


#[derive(Fsm)]
struct PdProtocolDefinition(
    ContextType<PdProtocolContext>,
    InitialState<PdProtocol, (ReceiveHeader, ReceivingData)>,

    TransitionInternalGuard < PdProtocol, ReceiveHeader, ByteReceived,                  OnByteReceived, HeaderDataRequired >,
    TransitionGuard         < PdProtocol, ReceiveHeader, ByteReceived, ReceiveData,     OnByteReceived, SomeDataRequired >,
    TransitionGuard         < PdProtocol, ReceiveHeader, ByteReceived, ReceiveCrc,      OnByteReceived, SomeDataNotRequired >,

    TransitionInternalGuard < PdProtocol, ReceiveData,   ByteReceived,                  OnByteReceived, MoreDataRequired >,
    Transition              < PdProtocol, ReceiveData,   ByteReceived, ReceiveCrc,      OnByteReceived >,
    
    TransitionInternalGuard < PdProtocol, ReceiveCrc,    ByteReceived,                  OnByteReceived, MoreCrcRequired >,
    Transition              < PdProtocol, ReceiveCrc,    ByteReceived, MessageReceived, OnByteReceived >,

    Transition < PdProtocol, ReceivingData, ErrorOccurred, ProtocolError, NoAction >,
    Transition < PdProtocol, ReceivingData, ToMessageDecoded, MessageDecoded, ToMessageDecoded >,

    InterruptState < PdProtocol, ProtocolError, ErrorOccurred >
);

#[test]
fn test_fusb302_traffic() {
    let traffic = [
        224, 97,
        71, 44, 145, 1, 8, 44, 209, 2, 8, 44, 177, 4, 8, 44, 65, 6, 8, 201, 183, 153, 250, 224, 97, 73, 
        
        44, 145, 1, 8, 44, 
        
        209, 2, 
        
        8, 44, 177, 4, 8, 44, 65, 6, 8, 95, 79, 189, 187, 224, 97, 75, 44, 145, 1, 8, 44, 209, 2, 8, 44, 177, 4, 8, 44, 65, 6, 8, 152, 223, 129, 111];


    let mut decoder = PdProtocol::new(Default::default()).unwrap();
    decoder.start();
    for &b in &traffic[..] {
        decoder.process_event(ByteReceived { byte: b });
    }
}

#[test]
fn test_lecroy_traffic() {
    let traffic = [
        0x1, 0x6, 0x1, 0x1, 
        0x0, 0xF, 0x0, 0x9, 
        0x1, 0x0, 0x8, 0x0, 
        0xA, 0x4, 0x4, 0xD, 
        0xF, 0x5, 0x3, 0x4 
    ];

    let traffic: Vec<_> = traffic.chunks(2).map(|c| {
        (c[1] as u8) << 4 | (c[0] as u8)
    }).collect();


    let mut decoder = PdProtocol::new(Default::default()).unwrap();
    decoder.start();
    for b in traffic {
        decoder.process_event(ByteReceived { byte: b }).unwrap();
        println!("state: {:?}", decoder.get_current_state());        
    }

    {
        let state: &ReceiveData = decoder.get_state();
        println!("data state: {:?}", state);

        let state: &ReceiveCrc = decoder.get_state();
        println!("crc state: {:?}", state);

        let state: &MessageDecoded = decoder.get_state();
        println!("message decoded state: {:?}", state);
        assert!(state.message.is_some());
    }

    //println!("decoder: {:?}", decoder);
}