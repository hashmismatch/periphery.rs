use prelude::v1::*;
use base::*;
use device_storage::*;

pub struct StorageUtils;

impl StorageUtils {
    pub fn erase_all<D: StorageDevice>(device: &D) -> Result<u64, PeripheryError> {
        let size_bytes = device.get_total_capacity_bytes();
        let erase = try!(device.get_sector_erase().ok_or(PeripheryError::NotImplemented));

        let mut offset = 0;
        let s = erase.get_erase_sector_size_bytes();
        loop {
            try!(erase.erase_sector(offset));

            offset += s;

            if s >= size_bytes { break; }
        }

        Ok(offset)
    }
}
