//! A high level abstraction for accessing memory devices like flash,
//! SD cards or EEPROMs.

use prelude::v1::*;
use base::*;
use bus::*;
use system::*;
use device::*;

pub fn unit_bytes_to_addressing_mask(unit: u64) -> Option<u64> {
    match unit {
        0x10 | 0x20 | 0x40 | 0x80 |
        0x100 | 0x200 | 0x400 | 0x800 |
        0x1000 | 0x2000 | 0x4000 | 0x8000 |
        0x10000 | 0x20000 | 0x40000 | 0x80000
          => Some(!(unit - 1)),
        _ => None
    }
}

#[test]
fn test_mask() {
    assert_eq!(Some(!0xFF), unit_bytes_to_addressing_mask(0x100));
    assert_eq!(Some(!0x1FF), unit_bytes_to_addressing_mask(0x200));
    assert_eq!(Some(!0xFFF), unit_bytes_to_addressing_mask(0x1000));
}

use utils::sectors::*;

pub struct RangeOp {
    buffer: Vec<u8>,
    iterator: OffsetIterator
}

pub trait StorageDevice {
    fn get_sector_erase(&self) -> Option<&StorageDeviceSectorErase> {
        None
    }

    fn get_total_capacity_bytes(&self) -> u64;
    
    fn get_read_sector_size_bytes(&self) -> u64;
    fn get_total_read_sectors(&self) -> u64 {
        self.get_total_capacity_bytes() / self.get_read_sector_size_bytes()
    }

    fn get_write_sector_size_bytes(&self) -> u64;

    fn get_read_sector_address_mask(&self) -> u64 {
        unit_bytes_to_addressing_mask(self.get_read_sector_size_bytes()).unwrap()
    }

    fn get_write_sector_address_mask(&self) -> u64 {
        unit_bytes_to_addressing_mask(self.get_write_sector_size_bytes()).unwrap()
    }
    
    fn range_op(&self, address: u64, read_len: usize) -> Result<RangeOp, PeripheryError> {
        let sector_size_bytes = self.get_read_sector_size_bytes();
        
        match SectorOffsetUtils::new(sector_size_bytes, self.get_total_read_sectors()) {
            Ok(s) => {
                match s.offset_range_to_sectors(&(address..(address + read_len as u64))) {
                    Ok(i) => {
                        Ok(RangeOp {
                            buffer: vec![0; sector_size_bytes as usize],
                            iterator: i
                        })
                    },
                    Err(_) => {
                        return Err(PeripheryError::BufferLengthError);
                    }
                }
            },
            Err(_) => {
                return Err(PeripheryError::Unknown);
            }
        }
    }

    fn read_sector(&self, address: u64, out: &mut [u8]) -> Result<(), PeripheryError>;
    fn write_sector(&self, address: u64, buf: &[u8]) -> Result<(), PeripheryError>;

    fn read_range(&self, address: u64, out: &mut [u8]) -> Result<(), PeripheryError> {
        let mut op = try!(self.range_op(address, out.len()));

        for sector in op.iterator {
            try!(self.read_sector(sector.sector * self.get_read_sector_size_bytes(), &mut op.buffer[..]));
            let slice = sector.get_sector_slice();
            
            let out_index = slice.get_source_content().clone();
            let mut out = out.index_mut(out_index);
            out.copy_from_slice(op.buffer.index(sector.sector_offset));
        }

        Ok(())
    }

    fn write_range(&self, address: u64, buf: &[u8]) -> Result<(), PeripheryError> {
        if self.get_read_sector_size_bytes() != self.get_write_sector_size_bytes() {
            return Err(PeripheryError::BufferLengthError);
        }

        let mut op = try!(self.range_op(address, buf.len()));

        for sector in op.iterator {
            let slice = sector.get_sector_slice();
            if let SourceSlice::Full { .. } = slice {
                // reading the sector beforehand isn't necessary
            } else {
                try!(self.read_sector(sector.sector * self.get_read_sector_size_bytes(), &mut op.buffer[..]));
            }
            
            let slice = sector.get_sector_slice();
            {
                let in_index = slice.get_source_content().clone();                
                let mut b = op.buffer.index_mut(sector.sector_offset);
                b.copy_from_slice(buf.index(in_index));
            }

            try!(self.write_sector(sector.sector * self.get_write_sector_size_bytes(), &op.buffer[..]));
        }

        Ok(())
    }

    fn read_iter_bytes<'a>(&'a self) -> Result<StorageDeviceReadBytesIterator<'a, Self>, PeripheryError> where Self: Sized {
        self.read_iter_bytes_with_offset(0)
    }

    fn read_iter_bytes_with_offset<'a>(&'a self, offset: u64) -> Result<StorageDeviceReadBytesIterator<'a, Self>, PeripheryError> where Self: Sized {
        let b = self.get_total_capacity_bytes();
        let s = self.get_read_sector_size_bytes();

        let mut buffer = vec![0; s as usize];

        let first_sector = offset / s;
        let sector_offset = offset % s;
        
        try!(self.read_sector(first_sector * s, &mut buffer[..]));

        Ok(StorageDeviceReadBytesIterator {
            device: self,
            bytes_remaining: b - offset,
            buffer: buffer,
            buffer_pos: sector_offset as usize,
            address: s * (first_sector + 1)
        })
    }
}

pub struct StorageDeviceReadBytesIterator<'a, D: 'a> {
    device: &'a D,
    bytes_remaining: u64,
    buffer: Vec<u8>,
    buffer_pos: usize,
    address: u64
}

impl<'a, D> Iterator for StorageDeviceReadBytesIterator<'a, D> where D: StorageDevice {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.bytes_remaining == 0 {
            return None;
        }

        if self.buffer_pos == self.buffer.len() {
            if let Ok(_) = self.device.read_sector(self.address, &mut self.buffer[..]) {
                self.buffer_pos = 0;
                self.address += self.buffer.len() as u64;
            } else {
                return None;
            }
        }

        let r = Some(self.buffer[self.buffer_pos]);

        self.bytes_remaining -= 1;
        self.buffer_pos += 1;
        
        r
    }
}

pub trait StorageDeviceSectorErase {
    fn get_erase_sector_size_bytes(&self) -> u64;
    
    fn get_erase_sector_address_mask(&self) -> u64 {
        unit_bytes_to_addressing_mask(self.get_erase_sector_size_bytes()).unwrap()
    }

    fn erase_sector(&self, address: u64) -> Result<(), PeripheryError>;
}