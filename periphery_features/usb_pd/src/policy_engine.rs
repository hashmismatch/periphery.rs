use fsm::*;

use std::fmt::Debug;

use packed_struct::*;
use message_decoder::*;
use structs::*;

pub trait MessageTransmit {
    fn send_message(&self, msg: &PdMessage) -> Result<(), ()>;
    fn send_hard_reset(&self) -> Result<(), ()>;
    fn protocol_reset(&self) -> Result<(), ()>;
    fn is_vbus_active(&self) -> bool;
    fn detect_orientation(&self);
}

pub trait Environment: Debug {
    type Transmit: MessageTransmit;

    fn get_transmit(&self) -> &Self::Transmit;
}

#[derive(Copy, Clone, Debug)]
pub enum PowerSupplyDemand {
    Exact(ExactPowerSupplyDemand)
}

#[derive(Copy, Clone, Debug)]
pub struct ExactPowerSupplyDemand {
    pub voltage_volts: f32,
    pub amps: f32
}


#[derive(Debug, Serialize)]
pub struct PolicyEngineContext<Env: Environment> /* where Env: Environment */ {
    #[serde(skip_serializing)]
    pub environment: Env,

    pub next_message_id: u8,
    pub source_capabilities: Option<PdSourceCapabilities>
}

impl<Env> PolicyEngineContext<Env> where Env: Environment {
    pub fn new(environment: Env) -> Self {
        PolicyEngineContext {
            environment: environment,
            
            next_message_id: 0,
            source_capabilities: None
        }
    }
}



// events
#[derive(Clone, Debug, Serialize)]
pub struct SendMessage {
    pub message: PdMessage
}
impl FsmEvent for SendMessage {}

#[derive(Clone, Debug, Serialize)]
pub struct MessageReceived {
    pub message: PdMessage
}
impl FsmEvent for MessageReceived {}

#[derive(Clone, Debug, Serialize)]
pub struct MessageSent {
    pub message: PdMessage
}
impl FsmEvent for MessageSent {}

#[derive(Copy, Clone, Debug, Serialize)]
pub struct GoodCrcReceived { pub message_id: usize }
impl FsmEvent for GoodCrcReceived {}

fsm_event_unit!(CrcReceiveTimedOut);
fsm_event_unit!(NoResponseTimedOut);
fsm_event_unit!(SinkWaitCapabilitiesTimedOut);
fsm_event_unit!(PsTransitionTimedOut);

fsm_event_unit!(VbusDetected);
fsm_event_unit!(VbusLost);

#[derive(Clone, Debug, Serialize)]
pub struct SourceCapabilitiesReceived {
    pub capabilities: PdSourceCapabilities
}
impl FsmEvent for SourceCapabilitiesReceived {}

#[derive(Clone, Debug, Serialize)]
pub struct SinkRequestPowerCapability {
    pub request_obj_pos: u8,
    pub capability_mismatch: bool,
    pub operating_current: u16,
    pub maximum_operating_current: u16
}
impl FsmEvent for SinkRequestPowerCapability {}

fsm_event_unit!(ProtocolLayerResetFinished);
fsm_event_unit!(HardResetComplete);
fsm_event_unit!(HardResetSent);
fsm_event_unit!(SinkDefaultsSet);


#[derive(Default, Debug, Serialize)]
pub struct ProtocolIdle { }
impl<Env: Environment> FsmState<PolicyEngine<Env>> for ProtocolIdle { }

#[derive(Default, Debug, Serialize)]
pub struct ProtocolWaitingForReply {
    pub expected_good_crc_message_id: Option<u8>
}
impl<Env: Environment> FsmState<PolicyEngine<Env>> for ProtocolWaitingForReply { }
impl<Env: Environment> StateTimeout<PolicyEngine<Env>> for ProtocolWaitingForReply {
    fn timeout_on_entry(&self, event_context: &mut EventContext<PolicyEngine<Env>>) -> Option<TimerSettings> {
        Some(TimerSettings {
            timeout: TimerDuration::from_millis(3000),
            cancel_on_state_exit: true 
        })
    }
}


#[derive(Default, Debug, Serialize)]
pub struct SinkStartup { }
impl<Env: Environment> FsmState<PolicyEngine<Env>> for SinkStartup {
    fn on_entry(&mut self, event_context: &mut EventContext<PolicyEngine<Env>>) {
        /* On entry to this state the Policy Engine Shall reset the Protocol Layer. Note that resetting the Protocol Layer will also reset the MessageIDCounter and stored MessageID (see Section 6.9.2.3).
            Once the reset process completes, the Policy Engine Shall transition to the PE_SNK_Discovery state for a Consumer only and to the PE_DB_CP_Check_for_VBUS state for a USB Type-B Consumer/Provider
        */

        if let Ok(_) = event_context.context.environment.get_transmit().protocol_reset() {
            println!("protocol reset?");
            event_context.queue.enqueue_event(ProtocolLayerResetFinished.into());
        }
    }
}



#[derive(Default, Debug, Serialize)]
pub struct SinkDiscovery { }
impl<Env: Environment> FsmState<PolicyEngine<Env>> for SinkDiscovery {
    fn on_entry(&mut self, event_context: &mut EventContext<PolicyEngine<Env>>) {
        if event_context.context.environment.get_transmit().is_vbus_active() {
            event_context.context.environment.get_transmit().detect_orientation();
            event_context.queue.enqueue_event(VbusDetected.into());
        }
    }
}
impl<Env: Environment> StateTimeout<PolicyEngine<Env>> for SinkDiscovery {
    fn timeout_on_entry(&self, event_context: &mut EventContext<PolicyEngine<Env>>) -> Option<TimerSettings> {
        Some(TimerSettings {
            timeout: TimerDuration::from_millis(5500),
            cancel_on_state_exit: true 
        })
    }
}

#[derive(Default, Debug, Serialize)]
pub struct SinkWaitForCapabilities { }
impl<Env: Environment> FsmState<PolicyEngine<Env>> for SinkWaitForCapabilities { }
impl<Env: Environment> StateTimeout<PolicyEngine<Env>> for SinkWaitForCapabilities {
    fn timeout_on_entry(&self, event_context: &mut EventContext<PolicyEngine<Env>>) -> Option<TimerSettings> {
        Some(TimerSettings {
            timeout: TimerDuration::from_millis(2500),
            cancel_on_state_exit: true 
        })
    }
}


#[derive(Default, Debug, Serialize)]
pub struct SinkEvaluateCapability { }
impl<Env: Environment> FsmState<PolicyEngine<Env>> for SinkEvaluateCapability {
    fn on_entry(&mut self, event_context: &mut EventContext<PolicyEngine<Env>>) {
        println!("evaluating caps!");

        if let Some(ref capabilities) = event_context.context.source_capabilities {
            let requested_voltage_volts = 9.00;
            let requested_amps = 2.0;
            let max_delta_volts = 0.1;

            //let mut obj_pos = None;
            //let mut mismatch = false;            

            for (i, obj) in capabilities.power.iter().enumerate() {
                let obj_pos = i + 1;
                match obj {
                    &PowerDataKind::Fixed(fixed) => {
                        let supply_volts = fixed.get_voltage_milli_volts() as f32 / 1000.0;
                        if (supply_volts - requested_voltage_volts).abs() <= max_delta_volts {
                            println!("found matching voltage");

                            let requested_current = *amps_to_usb_pd(requested_amps).unwrap();
                            println!("our desired current: {} (usb pd value)", requested_current);

                            let operating_current = requested_current.min(*fixed.maximum_current);

                            let (capability_mismatch, maximum_operating_current) = if requested_current <= *fixed.maximum_current {
                                (false, operating_current)
                            } else {
                                (true, requested_current)
                            };

                            let req = SinkRequestPowerCapability {
                                request_obj_pos: obj_pos as u8,
                                capability_mismatch: capability_mismatch,
                                operating_current: operating_current.into(),
                                maximum_operating_current: maximum_operating_current.into()
                            }.into();
                            
                            if capability_mismatch {
                                println!("the capability is mismatched. our request: {:#?}", &req);
                            }

                            event_context.queue.enqueue_event(PolicyEngineEvents::SinkRequestPowerCapability(req));                            

                            break;
                        }
                    },
                    _ => ()
                }
            }
        }
    }
}

#[derive(Default, Debug, Serialize)]
pub struct SinkSelectCapability { }
impl<Env: Environment> FsmState<PolicyEngine<Env>> for SinkSelectCapability { }

#[derive(Default, Debug, Serialize)]
pub struct SinkTransitionSink { }
impl<Env: Environment> FsmState<PolicyEngine<Env>> for SinkTransitionSink { }
impl<Env: Environment> StateTimeout<PolicyEngine<Env>> for SinkTransitionSink {
    fn timeout_on_entry(&self, event_context: &mut EventContext<PolicyEngine<Env>>) -> Option<TimerSettings> {
        Some(TimerSettings {
            timeout: TimerDuration::from_millis(500),
            cancel_on_state_exit: true 
        })
    }
}

#[derive(Default, Debug, Serialize)]
pub struct SinkReady { }
impl<Env: Environment> FsmState<PolicyEngine<Env>> for SinkReady { }

#[derive(Default, Debug, Serialize)]
pub struct SinkHardReset {
    pub hard_reset_counter: u8
}
impl<Env: Environment> FsmState<PolicyEngine<Env>> for SinkHardReset {
    fn on_entry(&mut self, event_context: &mut EventContext<PolicyEngine<Env>>) {
        self.hard_reset_counter += 1;
        event_context.context.environment.get_transmit().send_hard_reset();
        println!("hard reset counter: {}", self.hard_reset_counter);
        event_context.context.next_message_id = 0;
    }
}
impl<Env: Environment> StateTimeout<PolicyEngine<Env>> for SinkHardReset {
    fn timeout_on_entry(&self, event_context: &mut EventContext<PolicyEngine<Env>>) -> Option<TimerSettings> {
        Some(TimerSettings {
            timeout: TimerDuration::from_millis(50),
            cancel_on_state_exit: true 
        })
    }
}

#[derive(Default, Debug, Serialize)]
pub struct SinkTransitionToDefault { }
impl<Env: Environment> FsmState<PolicyEngine<Env>> for SinkTransitionToDefault {
    fn on_entry(&mut self, event_context: &mut EventContext<PolicyEngine<Env>>) {
        println!("transtion to default, delay required, our PD reset required?");
        event_context.queue.enqueue_event(SinkDefaultsSet.into());
    }
}

pub struct StoreSourceCapabilities;
impl<Env: Environment> FsmAction<PolicyEngine<Env>, SinkWaitForCapabilities, SourceCapabilitiesReceived, SinkEvaluateCapability> for StoreSourceCapabilities {
    fn action(event: &SourceCapabilitiesReceived, event_context: &mut EventContext<PolicyEngine<Env>>, source_state: &mut SinkWaitForCapabilities, target_state: &mut SinkEvaluateCapability) {
        event_context.context.source_capabilities = Some(event.capabilities.clone());
    }
}




pub struct StartSendingMessage;
impl<Env: Environment> FsmActionSelf<PolicyEngine<Env>, ProtocolIdle, SendMessage> for StartSendingMessage {
    fn action(event: &SendMessage, event_context: &mut EventContext<PolicyEngine<Env>>, state: &mut ProtocolIdle) {
    
        let mut msg = event.message.clone();
        let next_message_id = event_context.context.next_message_id;
        match msg.header {
            DecodedMessageHeader::Data(ref mut header) => {
                header.message_id = next_message_id.into();
            },
            DecodedMessageHeader::Control(ref mut header) => {
                header.message_id = next_message_id.into();
            }
        }
        
        match event_context.context.environment.get_transmit().send_message(&msg) {
            Ok(_) => {
                event_context.queue.enqueue_event(MessageSent { message: msg }.into());
            },
            Err(e) => {
                println!("Error sending message: {:?}", e);
            }
        }
        
    }
}


pub struct StartWaitingForMessage;
impl<Env: Environment> FsmAction<PolicyEngine<Env>, ProtocolIdle, MessageSent, ProtocolWaitingForReply> for StartWaitingForMessage {
    fn action(event: &MessageSent, event_context: &mut EventContext<PolicyEngine<Env>>, source_state: &mut ProtocolIdle, target_state: &mut ProtocolWaitingForReply) {
        target_state.expected_good_crc_message_id = Some(event.message.get_message_id());
        println!("stored source capabilities");
    }
}

impl<Env: Environment> FsmAction<PolicyEngine<Env>, SinkEvaluateCapability, SinkRequestPowerCapability, SinkSelectCapability> for SinkRequestPowerCapability {
    fn action(event: &SinkRequestPowerCapability, event_context: &mut EventContext<PolicyEngine<Env>>, source_state: &mut SinkEvaluateCapability, target_state: &mut SinkSelectCapability) {
    
        let data = PdData::Request(
            PdRequest::FixedAndVariable(FixedAndVariableRequest {
                object_position: event.request_obj_pos.into(),
                give_back: false,
                capability_mismatch: event.capability_mismatch,
                usb_communications_capable: false,
                no_usb_suspend: true,
                operating_current: 44.into(),
                maximum_operating_current: 44.into()
            })
        );

        let event = SendMessage {
            message: PdMessage::new_data(DataMessageHeader {
                number_of_data_objects: 1.into(),
                message_id: 0.into(),
                port_power_role: PortPowerRole::Sink,
                specification_revision: SpecificationRevision::Revision2,
                port_data_role: PortDataRole::Ufp,
                message_type: DataMessageType::Request
            }, data)
        };

        event_context.queue.enqueue_event(event.into());
    }
}

impl<Env: Environment> FsmActionSelf<PolicyEngine<Env>, ProtocolWaitingForReply, MessageReceived> for MessageReceived {
    fn action(event: &MessageReceived, event_context: &mut EventContext<PolicyEngine<Env>>, state: &mut ProtocolWaitingForReply) {
        println!("[XXX] message received: {:?}", event);
    }
}

pub struct IncrementNextMessageId;
impl<Env: Environment> FsmAction<PolicyEngine<Env>, ProtocolWaitingForReply, MessageReceived, ProtocolIdle> for IncrementNextMessageId {
    fn action(event: &MessageReceived, event_context: &mut EventContext<PolicyEngine<Env>>, source_state: &mut ProtocolWaitingForReply, target_state: &mut ProtocolIdle) {
        event_context.context.next_message_id = (event_context.context.next_message_id + 1) % 8;
    }
}

pub struct GoodCrcCheck;
impl<Env: Environment> FsmGuard<PolicyEngine<Env>, MessageReceived> for GoodCrcCheck {
    fn guard(event: &MessageReceived, event_context: &EventContext<PolicyEngine<Env>>, states: &PolicyEngineStatesStore) -> bool {
        if let &DecodedMessageHeader::Control(h) = &event.message.header {
            if h.message_type == ControlMessageType::GoodCrc && event_context.context.next_message_id == event.message.get_message_id()
            {
                return true;
            }
        }

        false
    }
}

pub struct ReceivedAccept;
impl<Env: Environment> FsmGuard<PolicyEngine<Env>, MessageReceived> for ReceivedAccept {
    fn guard(event: &MessageReceived, event_context: &EventContext<PolicyEngine<Env>>, states: &PolicyEngineStatesStore) -> bool {
        if let &DecodedMessageHeader::Control(h) = &event.message.header {
            if h.message_type == ControlMessageType::Accept {
                return true;
            }
        }

        false
    }
}

pub struct ReceivedPowerSupplyReady;
impl<Env: Environment> FsmGuard<PolicyEngine<Env>, MessageReceived> for ReceivedPowerSupplyReady {
    fn guard(event: &MessageReceived, event_context: &EventContext<PolicyEngine<Env>>, states: &PolicyEngineStatesStore) -> bool {
        if let &DecodedMessageHeader::Control(h) = &event.message.header {
            if h.message_type == ControlMessageType::PsRdy {
                return true;
            }
        }

        false
    }
}

impl<Env: Environment> FsmActionSelf<PolicyEngine<Env>, ProtocolIdle, MessageReceived> for MessageReceived {
    fn action(event: &MessageReceived, event_context: &mut EventContext<PolicyEngine<Env>>, state: &mut ProtocolIdle) {
        println!("[YYY] message received: {:?}", event);            
        
        match event.message.data {
            PdData::SourceCapabilities(ref caps) => {                    
                event_context.queue.enqueue_event(SourceCapabilitiesReceived { capabilities: caps.clone() }.into());
                println!("source caps received");
            },
            _ => ()
        }
    }
}


impl<Env: Environment> FsmAction<PolicyEngine<Env>, ProtocolWaitingForReply, CrcReceiveTimedOut, ProtocolIdle> for CrcReceiveTimedOut {
    fn action(event: &CrcReceiveTimedOut, event_context: &mut EventContext<PolicyEngine<Env>>, source_state: &mut ProtocolWaitingForReply, target_state: &mut ProtocolIdle) {
        println!("CRC receive timed out!");
    }
}


type P<Env> = PolicyEngine<Env>;

use fsm::console_inspect::*;

#[derive(Fsm)]
struct PolicyEngineDefinition<Env: Environment>(
    ContextType<PolicyEngineContext<Env>>,
    InitialState< P<Env>, (ProtocolIdle, SinkStartup)>,

    // protocol

    TransitionInternal < P<Env>, ProtocolIdle, SendMessage, StartSendingMessage >,    
    
    Transition < P<Env>, ProtocolIdle, MessageSent, ProtocolWaitingForReply, StartWaitingForMessage >,
    TimerStateTimeout < P<Env>, ProtocolWaitingForReply, CrcReceiveTimedOut >,
    Transition < P<Env>, ProtocolWaitingForReply, CrcReceiveTimedOut, ProtocolIdle, CrcReceiveTimedOut >,

    // good crc match
    TransitionGuard < P<Env>, ProtocolWaitingForReply, MessageReceived, ProtocolIdle, IncrementNextMessageId, GoodCrcCheck >,

    TransitionInternal < P<Env>, ProtocolWaitingForReply, MessageReceived, MessageReceived >,
    TransitionInternal < P<Env>, ProtocolIdle, MessageReceived, MessageReceived >,

    // policy

    Transition < P<Env>, SinkStartup, ProtocolLayerResetFinished, SinkDiscovery, NoAction >,


    Transition < P<Env>, SinkDiscovery, VbusDetected, SinkWaitForCapabilities, NoAction >,
    Transition < P<Env>, SinkDiscovery, NoResponseTimedOut, SinkHardReset, NoAction >,    
    TimerStateTimeout < P<Env>, SinkDiscovery, NoResponseTimedOut >,


    Transition < P<Env>, SinkWaitForCapabilities, SourceCapabilitiesReceived, SinkEvaluateCapability, StoreSourceCapabilities >,
    Transition < P<Env>, SinkWaitForCapabilities, SinkWaitCapabilitiesTimedOut, SinkHardReset, NoAction >,
    TimerStateTimeout < P<Env>, SinkWaitForCapabilities, SinkWaitCapabilitiesTimedOut >,


    Transition < P<Env>, SinkEvaluateCapability, SinkRequestPowerCapability, SinkSelectCapability, SinkRequestPowerCapability >,

    TransitionGuard < P<Env>, SinkSelectCapability, MessageReceived, SinkTransitionSink, NoAction, ReceivedAccept >,
    Transition < P<Env>, SinkTransitionSink, PsTransitionTimedOut, SinkHardReset, NoAction >,
    TransitionGuard < P<Env>, SinkTransitionSink, MessageReceived, SinkReady, NoAction, ReceivedPowerSupplyReady >,
    TimerStateTimeout< P<Env>, SinkTransitionSink, PsTransitionTimedOut >,

    Transition < P<Env>, SinkReady, VbusLost, SinkStartup, NoAction >,

    // todo: not really sure if we need to honor the interrupt from the usb pd
    TransitionInternal < P<Env>, SinkHardReset, HardResetSent, NoAction >,
    Transition < P<Env>, SinkHardReset, HardResetComplete, SinkTransitionToDefault, NoAction >,
    TimerStateTimeout < P<Env>, SinkHardReset, HardResetComplete >,
    Transition < P<Env>, SinkTransitionToDefault, SinkDefaultsSet, SinkStartup, NoAction >

    //Transition < P<Env>, SinkEvaluateCapability, 
	
);

