use prelude::v1::*;
use base::*;
use bus::*;
use register::*;
use system::*;

pub trait DeviceRegisterBus : Send + Sync {
    fn read_from_register(&self, register: u8, data: &mut [u8]) -> Result<(), PeripheryError>;
    fn write_to_register(&self, register: u8, data: &[u8]) -> Result<(), PeripheryError>;
    
}

#[derive(Copy, Clone, Debug)]
pub struct DeviceRegisterBusNotImplemented;
impl DeviceRegisterBus for DeviceRegisterBusNotImplemented {
    fn read_from_register(&self, register: u8, data: &mut [u8]) -> Result<(), PeripheryError> {
        Err(PeripheryError::NotImplemented)
    }
    fn write_to_register(&self, register: u8, data: &[u8]) -> Result<(), PeripheryError> {
        Err(PeripheryError::NotImplemented)
    }
}

pub trait DeviceCommandBus: Send + Sync {
    fn execute_command(&self, data: &[u8]) -> Result<(), PeripheryError>;
}

#[derive(Copy, Clone, Debug)]
pub struct DeviceCommandBusNotImplemented;
impl DeviceCommandBus for DeviceCommandBusNotImplemented {
    fn execute_command(&self, data: &[u8]) -> Result<(), PeripheryError> {
        Err(PeripheryError::NotImplemented)
    }
}


pub trait DeviceDataTransfer: Send + Sync {
    fn transmit(&self, data: &[u8]) -> Result<(), PeripheryError>;
    fn receive(&self, data: &mut [u8]) -> Result<(), PeripheryError>;
}


#[derive(Copy, Clone, Debug)]
pub struct DeviceDataTransferNotImplemented;
impl DeviceDataTransfer for DeviceDataTransferNotImplemented {
    fn transmit(&self, data: &[u8]) -> Result<(), PeripheryError> {
        Err(PeripheryError::NotImplemented)
    }
    fn receive(&self, data: &mut [u8]) -> Result<(), PeripheryError> {
        Err(PeripheryError::NotImplemented)
    }
}