#[macro_use]
extern crate periphery_core;

extern crate packed_struct;

#[macro_use]
extern crate packed_struct_codegen;

use periphery_core::prelude::v1::*;

use packed_struct::prelude::*;

#[derive(Copy, Clone, Debug, PackedStruct)]
#[packed_struct(size_bytes="4", bit_numbering="msb0")]
pub struct LedPixel {
    #[packed_field(bits="3..7")]
    brightness: Integer<u8, packed_bits::Bits5>,
    blue: u8,
    green: u8,
    red: u8
}

impl LedPixel {
    /// Brightness is a 5 bit field; the range is 0-31.
    /// The colors are full 8 bits, 0-255.
    pub fn new(brightness: u8, red: u8, green: u8, blue: u8) -> Self {
        let brightness = min(31, brightness);
        LedPixel {
            brightness: brightness.into(),
            red: red,
            green: green,
            blue: blue
        }
    }
}


#[derive(Clone)]
pub struct Apa102<B> where B: DeviceDataTransfer {
    bus: B
}

impl<B> Apa102<B> where B: DeviceDataTransfer {
    pub fn new(bus: B) -> Self {
        Apa102 {
            bus: bus
        }
    }

    pub fn send(&self, pixels: &[LedPixel]) -> Result<(), PeripheryError> {
        let mut buffer = Vec::with_capacity((pixels.len() + 2) * 4);
        // start frame
        buffer.extend(&[0, 0, 0,0 ]);

        for pixel in pixels {
            let mut p = pixel.pack();
            p[0] |= 0b11100000;
            buffer.extend_from_slice(&p[..]);
        }

        // end frame
        buffer.extend(&[0xFF, 0xFF, 0xFF, 0xFF]);

        self.bus.transmit(&buffer)?;
        
        Ok(())
    }
}
