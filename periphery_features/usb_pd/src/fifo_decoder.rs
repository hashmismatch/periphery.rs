use protocol_decoder::*;
use message_decoder::*;
use structs::*;
use fsm::*;

#[derive(Copy, Clone, Debug)]
pub enum FifoError {
    MoreInputRequired,
    InvalidHeadOfMessage(ProtocolErrorKind),
    ProcessingError
}

pub struct FifoDecoder {
    buffer: Vec<u8>
}

impl FifoDecoder {
    pub fn new() -> Self {
        FifoDecoder {
            buffer: vec![]
        }
    }

    pub fn decode_single_message(data: &[u8]) -> Result<PdMessage, FifoError> {
        let mut decoder = Self::new();
        for &b in data {
            match decoder.received_byte(b) {
                Ok(msg) => {
                    return Ok(msg);
                },
                Err(FifoError::MoreInputRequired) => { continue; }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Err(FifoError::ProcessingError)
    }

    pub fn received_byte(&mut self, byte: u8) -> Result<PdMessage, FifoError> {
        let mut decoder = PdProtocol::new(Default::default()).unwrap();
        decoder.start();

        self.buffer.push(byte);

        #[derive(Debug)]
        enum ActionAfter { NoAction, RemoveHead, MessageReceived };

        let mut action = ActionAfter::NoAction;
        
        for &b in &self.buffer {
            match decoder.process_event(ByteReceived { byte: b }) {
                Ok(_) => (),
                Err(e) => {
                    action = ActionAfter::RemoveHead;
                    break;
                }
            }

            match decoder.get_current_state() {
                (_, PdProtocolStates::ProtocolError) => {
                    // something is wrong with this starting byte, pop the head of the buffer
                    action = ActionAfter::RemoveHead;
                    break;
                },
                (_, PdProtocolStates::MessageDecoded) => {
                    // we got the message, clear the buffer
                    action = ActionAfter::MessageReceived;
                    break;
                },
                _ => {
                    // more input required                    
                }
            }
        }

        //println!("state: {:?}", decoder.get_current_state());

        match action {
            ActionAfter::RemoveHead => {
                self.buffer.remove(0);
                let s: &ProtocolError = decoder.get_state();
                Err(FifoError::InvalidHeadOfMessage(s.error.unwrap_or(ProtocolErrorKind::UnknownError)))
            },
            ActionAfter::MessageReceived => {
                self.buffer.clear();

                let state: &mut MessageDecoded = decoder.get_state_mut();
                if let Some(pd) = state.message.take() {
                    Ok(pd)
                } else {
                    Err(FifoError::ProcessingError)
                }
            },
            ActionAfter::NoAction => {
                Err(FifoError::MoreInputRequired)
            }
        }
    }
}



#[test]
fn fusb302_dump() {
    let traffic = [ 224, 97, 71, 44, 145, 1, 8, 44, 209, 2, 8, 44, 177, 4, 8, 44, 65, 6, 8, 201, 183, 153, 250, 224, 97, 73,  44, 145, 1, 8, 44, 209, 2, 8, 44, 177, 4, 8, 44, 65, 6, 8, 95, 79, 189, 187, 224, 97, 75, 44, 145, 1, 8, 44, 209, 2, 8, 44, 177, 4, 8, 44, 65, 6, 8, 152, 223, 129, 111];

    let mut fifo = FifoDecoder::new();
    for &b in &traffic[..] {
        match fifo.received_byte(b) {
            Ok(msg) => {
                println!("Message: {:#?}", msg);
            },
            Err(e) => {
                //println!("Error: {:?}", e);
            }
        }
        
        /*
        if let Ok(msg) = fifo.received_byte(b) {
            println!("Message: {:#?}", msg);
        } else {
            println!("stuff")
        }
        */
    }
}