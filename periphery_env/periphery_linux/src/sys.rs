use periphery_core::prelude::v1::*;
use periphery_core::*;

#[derive(Clone)]
pub struct StdSystemApi;
impl SystemApi for StdSystemApi {
	fn get_sleep(&self) -> Result<&SystemApiSleep, PeripheryError> {
		Ok(self)
	}

	fn get_debug(&self) -> Result<&SystemApiDebug, PeripheryError> {
		Ok(self)
	}
}

impl SystemApiSleep for StdSystemApi {
	fn sleep_ms(&self, ms: u32) {
		::std::thread::sleep_ms(ms);
	}
}

impl SystemApiDebug for StdSystemApi {
	fn debug(&self, line: &str) {
		println!("[DEBUG] {}", line);
	}
}  

use std::sync::{Arc, Mutex};

/*
#[derive(Clone)]
pub struct SharedLockedOperationsBus<S, B> {
	bus: Arc<B>,
	_system: PhantomData<S>,
	lock: Arc<Mutex<()>>
}

impl<S, B> SharedLockedOperationsBus<S, B> where B: Bus<S>, S: SystemApi {
	pub fn new(bus: B) -> Self {
		SharedLockedOperationsBus {
			bus: Arc::new(bus),
			_system: Default::default(),
			lock: Arc::new(Mutex::new(()))
		}
	}
}

impl<S, B> Bus<S> for SharedLockedOperationsBus<S, B> where B: Bus<S>, S: SystemApi {
    fn get_i2c(&self) -> Result<&I2CBus, PeripheryError> {
		Ok(self)
	}

    fn get_system_api(&self) -> &S {
		self.bus.get_system_api()
	}

    fn get_cli_prefix(&self) -> Result<Cow<str>, PeripheryError> {
        self.bus.get_cli_prefix()
    }
}

impl<S, B> I2CBus for SharedLockedOperationsBus<S, B> where B: Bus<S>, S: SystemApi {
    fn read(&self, device: I2CAddress, data: &mut [u8]) -> Result<(), PeripheryError> {
		if let Ok(_) = self.lock.lock() {
			return self.bus.get_i2c()?.read(device, data);
		}
		
		Err(PeripheryError::LockingError)
    }

	fn write(&self, device: I2CAddress, data: &[u8]) -> Result<(), PeripheryError> {
		if let Ok(_) = self.lock.lock() {
			return self.bus.get_i2c()?.write(device, data);
		}
		
		Err(PeripheryError::LockingError)
    }

    fn read_from_register(&self, device: I2CAddress, address: u8, data: &mut [u8]) -> Result<(), PeripheryError> {
        if let Ok(_) = self.lock.lock() {
			return self.bus.get_i2c()?.read_from_register(device, address, data);
		}
		
		Err(PeripheryError::LockingError)
    }

	fn write_to_register(&self, device: I2CAddress, address: u8, data: &[u8]) -> Result<(), PeripheryError> {
        if let Ok(_) = self.lock.lock() {
			return self.bus.get_i2c()?.write_to_register(device, address, data);
		}
		
		Err(PeripheryError::LockingError)
    }

	fn ping(&self, device: I2CAddress) -> Result<bool, PeripheryError> {
        if let Ok(_) = self.lock.lock() {
			return self.bus.get_i2c()?.ping(device);
		}
		
		Err(PeripheryError::LockingError)
    }
}
*/