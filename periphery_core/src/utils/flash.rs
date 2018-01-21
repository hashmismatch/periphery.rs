use prelude::v1::*;
use base::*;
use device_storage::*;

#[derive(Copy, Clone)]
pub struct FlashUtils {
    pub empty_byte: u8
}

#[derive(Clone, Debug)]
pub enum FindSpaceError {
    DeviceTooSmall,
    EmptyRequest,
    InvalidOffset,
    OutOfFreeSpace,
    System(PeripheryError)
}

impl FlashUtils {
    /// Linearly and naively scans the storage for empty space upwards, defined by the "empty_byte" field.
    pub fn find_empty_space<D: StorageDevice>(&self, device: &D, required_bytes: u64, offset: Option<u64>, alignment: Option<u64>) -> Result<u64, FindSpaceError> {
        if required_bytes == 0 {
            return Err(FindSpaceError::EmptyRequest);
        }

        let total = device.get_total_capacity_bytes();
        if total < required_bytes {
            return Err(FindSpaceError::DeviceTooSmall);
        }

        let mut offset = offset.unwrap_or(0);
        if offset >= total {
            return Err(FindSpaceError::InvalidOffset);
        }

        let iter = match device.read_iter_bytes_with_offset(offset) {
            Ok(iter) => iter,
            Err(e) => { return Err(FindSpaceError::System(e)); }
        };

        let mut available_space = 0;        
        let mut available_space_offset = offset;
                
        for b in iter {
            let in_alignment = if let Some(a) = alignment {
                available_space != 0 || (offset % a) == 0
            } else {
                true
            };

            if b == self.empty_byte && in_alignment {
                available_space += 1;
            } else {
                available_space = 0;
                available_space_offset = offset + 1;
            }
            offset += 1;

            if available_space >= required_bytes {
                return Ok(available_space_offset);
            }

            if offset >= total {
                break;
            }
        }

        Err(FindSpaceError::OutOfFreeSpace)
    }
}
