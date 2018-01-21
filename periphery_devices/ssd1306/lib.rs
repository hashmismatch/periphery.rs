#[macro_use]
extern crate periphery_core;

extern crate packed_struct;
#[macro_use]
extern crate packed_struct_codegen;

use packed_struct::*;


use periphery_core::*;
use periphery_core::prelude::v1::*;
use periphery_core::terminal_cli::*;

use periphery_core::prelude::v1::commands::ChipCommand;

pub type Ssd1306OnI2CBus<B> =
    Ssd1306<
        <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::Commands,
        <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::DataTransfer
    >;


#[derive(PackedStruct, Copy, Clone, Default)]
pub struct AddressingRange {
    pub start: u8,
    pub end: u8
}

commands!(
  chip Ssd1306Commands {
    command [0x81; 1] => set_contrast: u8,
    
    command [0xAF; 0] => display_on: EmptyReg,
    command [0xAE; 0] => display_off: EmptyReg,
    command [0xA5; 0] => display_all_on: EmptyReg,
    command [0xA4; 0] => display_all_on_resume: EmptyReg,

    command [0x21; 2] => column_address: AddressingRange,
    command [0x22; 2] => page_address: AddressingRange,

    command [0x8D; 1] => charge_pump: u8,
    command [0xD5; 1] => display_clock_div: u8,
    command [0xA8; 1] => set_multiplex: u8,
    command [0x20; 1] => memory_mode: u8,
    command [0xA0; 0] => segremap: u8,
    command [0xA1; 0] => segremap_1: u8,

    command [0xC0; 0] => com_scan_inc: EmptyReg,
    command [0xC8; 0] => com_scan_dec: EmptyReg,
    command [0xDA; 1] => comp_ins: u8,
    command [0xD9; 1] => set_precharge: u8,
    command [0xA6; 0] => normal_display: EmptyReg,
    command [0xA7; 0] => invert_display: EmptyReg,
    command [0xD3; 1] => set_display_offset: u8,
    command [0x40; 0] => set_start_line: EmptyReg,
    command [0xDB; 1] => set_vcom_detect: u8,

    command [0x2E; 0] => stop_scroll: EmptyReg
  }
);


#[derive(Clone, Copy)]
pub struct Ssd1306Factory {
    addresses: [I2CAddress; 2]
}

impl Default for Ssd1306Factory {
    fn default() -> Self {
        Ssd1306Factory {
            addresses: [
                I2CAddress::address_7bit(0x3c),
                I2CAddress::address_7bit(0x3d)
            ]
        }
    }
}

impl<B> DeviceI2CDetection<Ssd1306OnI2CBus<B>, B, I2CDeviceAll<B>> for Ssd1306Factory 
    where B: Bus + 'static,
{
	fn get_addresses(&self) -> &[I2CAddress] {
        &self.addresses
    }

	fn new(args: I2CDeviceAll<B>) -> Result<Ssd1306OnI2CBus<B>, PeripheryError> {
        let lcd = Ssd1306 {
            bus_commands: args.device_commands,
            bus_data: args.device_data
        };

        // todo: somehow verify that the controller is responding
        
        Ok(lcd)
    }
}

#[derive(Clone)]
pub struct Ssd1306<C, D> where C: DeviceCommandBus, D: DeviceDataTransfer {
    bus_commands: C,
    bus_data: D
}

impl<C, D> Ssd1306<C, D> where C: DeviceCommandBus, D: DeviceDataTransfer {
    pub fn commands<'b>(&'b self) -> Ssd1306Commands<'b, C> {
        Ssd1306Commands::new(&self.bus_commands)
    }

    pub fn init(&self) -> Result<(), PeripheryError> {
        let c = self.commands();
        
        c.display_off().execute()?;
        c.display_clock_div().execute_args(0x80)?; // Increase speed of the display max ~96Hz
        c.set_multiplex().execute_args(0x1F)?;
        c.set_display_offset().execute_args(0x00)?;
        c.set_start_line().execute()?;
        c.charge_pump().execute_args(0x14)?;
        c.memory_mode().execute_args(0x00)?;
        c.segremap_1().execute()?;
        c.com_scan_dec().execute()?;
        c.comp_ins().execute_args(0x02)?;
        
        c.set_contrast().execute_args(0x8F)?;
        c.set_precharge().execute_args(0xF1)?;        
        c.set_vcom_detect().execute_args(0x40)?;

        //c.stop_scroll().execute()?;

        // clear the screen
        let d = BwDisplayBuffer::new_128_32();
        let res = self.display(&d);
                
        c.display_all_on_resume().execute()?;
        c.normal_display().execute()?;
        c.display_on().execute()?;

        Ok(())
    }    

    pub fn display(&self, buffer: &BwDisplayBuffer) -> Result<(), PeripheryError> {
        self.commands().column_address().execute_args(AddressingRange { start: 0, end: (buffer.width - 1) as u8 })?;
        self.commands().page_address().execute_args(AddressingRange { start: 0, end: ((buffer.height / 8) - 1) as u8 })?;

        panic!("todo display");

        /*
        match self.bus.get_address() {
			ChipOnBus::I2C { address } => {
				let i2c = self.bus.get_bus().get_i2c()?;

                /*
                for row in 0..buffer.height {
                    for p in 0..(buffer.width / 8) {
                        let mut buf = [0; 8];
                        buf[0] = 0x40;
                    }
                }
                */

                for chunk in buffer.data.chunks(16) {
                    let mut buf = [0; 17];
                    buf[0] = 0x40;
                    &mut buf[1..].copy_from_slice(chunk);

                    i2c.write(address, &buf)?;
                }
            },
            _ => {
                panic!("todo");
            }
        }
        */

        Ok(())
    }
}

pub struct BwDisplayBuffer {
    width: usize,
    height: usize,
    data: Vec<u8>
}

impl BwDisplayBuffer {
    pub fn new_128_32() -> Self {
        BwDisplayBuffer {
            width: 128,
            height: 32,
            data: vec![0x00; (128*32) / 8]
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, pixel_on: bool) {
        if x >= self.width || y >= self.height {
            return;
        }
        
        let idx = x + ((y/8)*self.width);
        
        if pixel_on {
            let p = 1 << (y & 7);
            self.data[idx] |= p;
        } else {
            let p = !(1 << (y & 7));
            self.data[idx] &= p;
        }
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }
}



impl<C, D> Device for Ssd1306<C, D> where C: DeviceCommandBus, D: DeviceDataTransfer {
    fn get_registers_cli(&self) -> Option<DeviceBusCli> {
        let mut c = DeviceBusCli::new();
        c.with_commands(self.commands());
        Some(c)
    }    

	fn description(&self) -> Cow<str> {
        "ssd1306".into()
    }

	fn id(&self) -> Cow<str> {
        "ssd1306".into()
    }

	fn init_after_detection(&self) -> Result<bool, PeripheryError> {
        self.init()?;
		Ok(true)
	}

	fn get_cli(&self) -> Option<&DeviceCli> {
		Some(self)
	}    
}

impl<C, D> DeviceCli for Ssd1306<C, D> where C: DeviceCommandBus, D: DeviceDataTransfer {
    fn execute_cli(&self, exec: &mut PrefixedExecutor) {
        if let Some(mut cmd) = exec.command(&"display/test1") {
            self.init();

            let mut d = BwDisplayBuffer::new_128_32();
            for i in 0..32 {
                d.set_pixel(i, i, true);
            }

            let res = self.display(&d);

            match res {
                Ok(_) => cmd.get_terminal().print_line("Data sent!"),
                Err(e) => { write!(cmd.get_terminal(), "Error: {:?}", e); }
            }
        }

        if let Some(mut cmd) = exec.command(&"display/clear") {
            self.init();

            let d = BwDisplayBuffer::new_128_32();            
            let res = self.display(&d);

            match res {
                Ok(_) => cmd.get_terminal().print_line("Data sent!"),
                Err(e) => { write!(cmd.get_terminal(), "Error: {:?}", e); }
            }
        }
    }
}


#[test]
#[cfg(test)]
fn test_buffer() {    
    let mut d = BwDisplayBuffer::new_128_32();
    assert_eq!(d.data[0], 0);

    d.set_pixel(0, 0, true);
    //assert_eq!(d.data[0], 0b10000000);
    d.set_pixel(7, 0, true);
    //assert_eq!(d.data[0], 0b10000001);
    d.set_pixel(0, 0, false);
    //assert_eq!(d.data[0], 0b00000001);

    d.set_pixel(127, 31, true);
    //assert_eq!(*d.data.iter().last().unwrap(), 1);
}