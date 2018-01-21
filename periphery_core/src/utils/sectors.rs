use prelude::v1::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SectorError {
    SectorTooSmall,
    OffsetInvalid
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SectorOffsetUtils {
	sector_size: u64,
	sectors: u64
}

impl SectorOffsetUtils {
	pub fn new(sector_size: u64, sectors: u64) -> Result<SectorOffsetUtils, SectorError> {
		if sector_size <= 1 || sectors == 0 { return Err(SectorError::SectorTooSmall); }

		Ok(SectorOffsetUtils {
			sector_size: sector_size,
			sectors: sectors
		})
	}

	pub fn offset_to_sector(&self, offset: u64) -> u64 {
		offset / self.sector_size
	}

	pub fn sector_to_offset(&self, sector: u64) -> u64 {
		sector * self.sector_size
	}

	pub fn offset_range_to_sectors(&self, offset: &Range<u64>) -> Result<OffsetIterator, SectorError> {
		if offset.start >= offset.end { return Err(SectorError::OffsetInvalid); }
		{
			let max_offset = self.sectors * self.sector_size;
			if offset.start >= max_offset || offset.end > max_offset {
				return Err(SectorError::OffsetInvalid);
			}
		}

		let current_offset = offset.start;
		Ok(OffsetIterator {
			offset_range: offset.clone(),
			utils: *self,
			current_offset: Some(current_offset)
		})
	}
}

#[derive(Debug, PartialEq)]
pub struct OffsetIterator {
	offset_range: Range<u64>,
	utils: SectorOffsetUtils,
	current_offset: Option<u64>
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OffsetIteratorItem {
	pub sector: u64,
	pub sector_offset: Range<usize>,
	pub offset: Range<u64>,
    pub offset_range: Range<u64>,
    pub sector_size_bytes: u64
}

impl OffsetIteratorItem {
	/*
	pub fn copy_with_offset(&self, sector_src: &[u8], dst: &mut [u8], dst_offset: usize) {
		let src = &sector_src[(self.sector_offset.start as usize)..(self.sector_offset.end as usize)];
		let dst = &mut dst[((self.offset.start as usize) - (dst_offset as usize))..((self.offset.end as usize) - (dst_offset as usize))];

		dst.copy_from_slice(src);
	}
	*/

    pub fn get_sector_slice(&self) -> SourceSlice {
        let o = self.offset.start - self.offset_range.start;
        let l = self.offset.end - self.offset.start;
        let c = (o as usize) .. ((o + l) as usize);
        

        if l == self.sector_size_bytes {
            SourceSlice::Full { source_content: c }
        } else if l < self.sector_size_bytes && self.sector_offset.start != 0 && self.sector_offset.end != self.sector_size_bytes as usize {
            SourceSlice::EmptyBothSides {
                sector_empty_beginning: 0..self.sector_offset.start as usize,
                sector_empty_end: self.sector_offset.end as usize..(self.sector_size_bytes) as usize,
                source_content: c
            }
        } else if self.offset.start == self.offset_range.start {            
            SourceSlice::EmptyBeginning {
                sector_empty: 0..(self.sector_size_bytes - l) as usize,
                source_content: c
            }
        } else {
            SourceSlice::EmptyEnd {
                sector_empty: l as usize..(self.sector_size_bytes) as usize,
                source_content: c
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SourceSlice {
    Full { source_content: Range<usize> },
    EmptyBeginning { sector_empty: Range<usize>, source_content: Range<usize> },
    EmptyEnd { sector_empty: Range<usize>, source_content: Range<usize> },
    EmptyBothSides { sector_empty_beginning: Range<usize>, sector_empty_end: Range<usize>, source_content: Range<usize> }
}

impl SourceSlice {
    pub fn get_source_content(&self) -> &Range<usize> {
        match self {
            &SourceSlice::Full { ref source_content } => source_content,
            &SourceSlice::EmptyBeginning { ref source_content, .. } => source_content,
            &SourceSlice::EmptyEnd { ref source_content, .. } => source_content,
            &SourceSlice::EmptyBothSides { ref source_content, .. } => source_content
        }
    }
}

impl Iterator for OffsetIterator {
    type Item = OffsetIteratorItem;

    fn next(&mut self) -> Option<OffsetIteratorItem> {
    	if self.current_offset == None { return None; }
    	let current_offset = self.current_offset.unwrap();

    	let sector_start = self.utils.offset_to_sector(current_offset);
    	let sector_end = sector_start + 1;

    	let sector_start_bytes = sector_start * self.utils.sector_size;
    	let _sector_end_bytes = sector_end * self.utils.sector_size;

    	let offset_start = max(sector_start * self.utils.sector_size, self.offset_range.start);
    	let offset_end = min(sector_end * self.utils.sector_size, self.offset_range.end);

    	let r = offset_end - offset_start;

    	let sector_offset_start = offset_start - sector_start_bytes;
    	let sector_offset_end = sector_offset_start + r;

    	if offset_end >= self.offset_range.end {
    		self.current_offset = None;
    	} else {
    		self.current_offset = Some(offset_end);
    	}

    	let i = OffsetIteratorItem {
    		sector: sector_start,
    		sector_offset: (sector_offset_start as usize)..(sector_offset_end as usize),
    		offset: offset_start..offset_end,
            offset_range: self.offset_range.clone(),
            sector_size_bytes: self.utils.sector_size
    	};

    	Some(i)
    }
}

#[cfg(test)]
#[test]
pub fn test_sector_utils() {
	use prelude::v1::*;


	let utils = SectorOffsetUtils::new(512, 8).unwrap();
	
	{
		let sectors: Vec<_> = utils.offset_range_to_sectors(&(1023..1025)).unwrap().collect();
		assert_eq!(&[OffsetIteratorItem { sector: 1, sector_offset: 511..512, offset: 1023..1024, offset_range: 1023..1025, sector_size_bytes: 512}, 
			         OffsetIteratorItem { sector: 2, sector_offset: 0..1, offset: 1024..1025, offset_range: 1023..1025, sector_size_bytes: 512}], sectors.as_slice());

        let slices: Vec<_> = sectors.iter().map(|x| x.get_sector_slice()).collect();        
        assert_eq!(&[SourceSlice::EmptyBeginning { sector_empty: 0..511, source_content: 0..1 },
                     SourceSlice::EmptyEnd { sector_empty: 1..512, source_content: 1..2 }],
                   &slices[..]);
	}

	{
		let sectors: Vec<_> = utils.offset_range_to_sectors(&(0..1024)).unwrap().collect();
		assert_eq!(&[OffsetIteratorItem { sector: 0, sector_offset: 0..512, offset: 0..512, offset_range: 0..1024, sector_size_bytes: 512},
			         OffsetIteratorItem { sector: 1, sector_offset: 0..512, offset: 512..1024, offset_range: 0..1024, sector_size_bytes: 512}], sectors.as_slice());
	}

    {
        let sectors: Vec<_> = utils.offset_range_to_sectors(&(0..1024)).unwrap().collect();
        let slices: Vec<_> = sectors.iter().map(|x| x.get_sector_slice()).collect();
        assert_eq!(&[SourceSlice::Full { source_content: 0..512 }, SourceSlice::Full { source_content: 512..1024 }], &slices[..]);
        
    }

	{
		let single_sector = SectorOffsetUtils::new(512, 1).unwrap();

		{
			let single_byte: Vec<_> = single_sector.offset_range_to_sectors(&(511..512)).unwrap().collect();
			assert_eq!(&[OffsetIteratorItem { sector: 0, sector_offset: 511..512, offset: 511..512, offset_range: 511..512, sector_size_bytes: 512}], single_byte.as_slice());
		}

        {
            let middle = single_sector.offset_range_to_sectors(&(10..20)).unwrap().next().unwrap();
            let sector = middle.get_sector_slice();
            assert_eq!(SourceSlice::EmptyBothSides { sector_empty_beginning: 0..10, sector_empty_end: 20..512, source_content: 0..10 }, sector);
        }

        {
            assert_eq!(Err(SectorError::OffsetInvalid), single_sector.offset_range_to_sectors(&(0..513)));
        }
	}
}

