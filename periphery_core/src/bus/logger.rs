//! Wrapper for bus implementations, with tracing via the system API.

use prelude::v1::*;
use terminal_cli::*;

use std::sync::{Arc, Mutex};

type Ctx<S> = LoggerContext<S>;

#[derive(Clone)]
pub struct LoggerContext<S> where S: SystemApi {
    inner: Arc<Mutex<LoggerContextInner<S>>>
}

pub struct LoggerContextInner<S> where S: SystemApi {
    system: S
}

impl<S> LoggerContext<S> where S: SystemApi {
    fn log(&self, message: &str) {
        if let Ok(ctx) = self.inner.lock() {
            ctx.system.trace(message);
        }
    }

    fn log_error(&self, error: &PeripheryError) {
        if let Ok(ctx) = self.inner.lock() {
            ctx.system.trace(&format!("Error: {:?}", error));
        }
    }
}

#[derive(Clone)]
pub struct Logger<B> where B: Bus {
    bus: B,
    ctx: Ctx<B::SystemApi>
}

impl<B> Logger<B> where B: Bus {
    pub fn new(bus: B) -> Self {
        let system = bus.get_system_api();
        Logger {
            bus: bus,
            ctx: 
                LoggerContext {
                    inner: Arc::new(Mutex::new(
                        LoggerContextInner {
                            system: system
                        }
                    ))
                }
        }
    }

    pub fn logger_cli(&self, exec: &mut CliExecutor) {
        if let Ok(cli_prefix) = self.bus.get_cli_prefix() {            
            let cmd = format!("{}/log/enable", cli_prefix);
            if let Some(mut ctx) = exec.command(&cmd) {

            }

            let cmd = format!("{}/log/disable", cli_prefix);
            if let Some(mut ctx) = exec.command(&cmd) {
                
            }
        }
    }
}

impl<B> Bus for Logger<B> where B: Bus {
    type SystemApi = B::SystemApi;
    type I2C = LoggerI2C<B::SystemApi, B::I2C>;
    type Spi = B::Spi;

	fn get_i2c(&self) -> Result<Self::I2C, PeripheryError> {
		Ok(LoggerI2C {
            i2c: self.bus.get_i2c()?,
            ctx: self.ctx.clone()
        })
	}

	fn get_spi(&self) -> Result<Self::Spi, PeripheryError> {
		self.bus.get_spi()
	}    

    fn get_system_api(&self) -> Self::SystemApi {
        self.bus.get_system_api()
    }

    fn get_cli_prefix(&self) -> Result<Cow<str>, PeripheryError> {
        self.bus.get_cli_prefix()
    }
}

#[derive(Clone)]
pub struct LoggerI2C<S, I> where S: SystemApi {
    i2c: I,
    ctx: Ctx<S>
}

impl<S, I> I2CBus for LoggerI2C<S, I> where S: SystemApi, I: I2CBus {
	type DeviceFactory = LoggerI2CBusDeviceFactory<S, I::DeviceFactory>;

	fn read(&self, device: I2CAddress, data: &mut [u8]) -> Result<(), PeripheryError> {
        self.i2c.read(device, data)
    }
	fn write(&self, device: I2CAddress, data: &[u8]) -> Result<(), PeripheryError> {
        self.i2c.write(device, data)
    }

	fn read_from_register(&self, device: I2CAddress, address: u8, data: &mut [u8]) -> Result<(), PeripheryError> {
        self.i2c.read_from_register(device, address, data)
    }
	fn write_to_register(&self, device: I2CAddress, address: u8, data: &[u8]) -> Result<(), PeripheryError> {
        self.i2c.write_to_register(device, address, data)
    }
	fn ping(&self, device: I2CAddress) -> Result<bool, PeripheryError> {
        self.i2c.ping(device)
    }
    
	fn new_device_factory(&self) -> Result<Self::DeviceFactory, PeripheryError> {
        Ok(LoggerI2CBusDeviceFactory {
            factory: self.i2c.new_device_factory()?,
            ctx: self.ctx.clone()
        })
    }
}

pub struct LoggerI2CBusDeviceFactory<S, F> where S: SystemApi {
    factory: F,
    ctx: Ctx<S>
}

impl<S, F> I2CBusDeviceFactory for LoggerI2CBusDeviceFactory<S, F> where S: SystemApi, F: I2CBusDeviceFactory {
	type Registers = LoggerDeviceRegisters<S, F::Registers>;
	type Commands = F::Commands;
	type DataTransfer = F::DataTransfer;

	fn new_i2c_device_registers(&self, address: I2CAddress) -> Result<Self::Registers, PeripheryError> {
		let bus = self.factory.new_i2c_device_registers(address)?;
        Ok(LoggerDeviceRegisters {
            bus: bus,
            ctx: self.ctx.clone(),
            address: address
        })
	}

	fn new_i2c_device_commands(&self, address: I2CAddress) -> Result<Self::Commands, PeripheryError> {
		self.factory.new_i2c_device_commands(address)
	}

	fn new_i2c_device_data_transfer(&self, address: I2CAddress) -> Result<Self::DataTransfer, PeripheryError> {
		self.factory.new_i2c_device_data_transfer(address)
	}    
}

pub struct LoggerDeviceRegisters<S, R> where S: SystemApi {
    bus: R,
    ctx: Ctx<S>,
    address: I2CAddress
}

impl<S, R> DeviceRegisterBus for LoggerDeviceRegisters<S, R> where S: SystemApi, R: DeviceRegisterBus {
    fn read_from_register(&self, register: u8, data: &mut [u8]) -> Result<(), PeripheryError> {
        self.ctx.log(&format!("I2C device {}, reading {} bytes from register 0x{:X}", self.address, data.len(), register));

        match self.bus.read_from_register(register, data) {
            Ok(o) => {
                self.ctx.log(&format!("Data received: {:?}", data));
                Ok(o)
            },
            Err(e) => {
                self.ctx.log_error(&e);
                Err(e)
            }
        }
    }

    fn write_to_register(&self, register: u8, data: &[u8]) -> Result<(), PeripheryError> {
        self.ctx.log(&format!("I2C device {}, writing to register 0x{:X}, data {:?}, {} bytes", self.address, register, data, data.len()));

        match self.bus.write_to_register(register, data) {
            Ok(o) => {
                self.ctx.log("Data written.");
                Ok(o)
            },
            Err(e) => {
                self.ctx.log_error(&e);
                Err(e)
            }
        }
    }
}