//! System API interface

use prelude::v1::*;
use base::*;

/// Interface for interaction with the operating system
pub trait SystemApi : Send + Sync + Clone {
	fn get_sleep(&self) -> Result<&SystemApiSleep, PeripheryError> {
		Err(PeripheryError::MissingSystemSleep)
	}

	fn get_debug(&self) -> Result<&SystemApiDebug, PeripheryError> {
		Err(PeripheryError::NotImplemented)
	}


	fn trace(&self, line: &str) {
		if let Ok(debug) = self.get_debug() {
			debug.debug(line);
		}
	}

	fn debug(&self, line: &str) {
		if let Ok(debug) = self.get_debug() {
			debug.debug(line);
		}
	}

	fn sleep_ms(&self, ms: u32) {
		if let Ok(sleep) = self.get_sleep() {
			sleep.sleep_ms(ms);
		}
	}
}

pub trait SystemApiSleep {
	fn sleep_ms(&self, ms: u32);
}

pub trait SystemApiDebug {
	fn debug(&self, line: &str);
}

