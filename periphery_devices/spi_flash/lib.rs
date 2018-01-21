#[macro_use]
extern crate periphery_core;

extern crate packed_struct;

#[macro_use]
extern crate packed_struct_codegen;

use periphery_core::*;
use periphery_core::prelude::v1::*;
use periphery_core::bus::spi::*;

use packed_struct::*;

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(endian="msb")]
pub struct JedecId {
    pub manufacturer: u8,
    pub memory_type: u8,
    pub memory_capacity: u8
}

pub const MFG_MICRON: u8 = 0x20;
pub const MFG_WINBOND: u8 = 0xEF;
pub const MFG_MACRONIX: u8 = 0xC2;

#[derive(Clone, Copy, Debug)]
struct Geometry {
    page_size_bytes: usize,
    pages_per_sector: usize,
    sectors: usize
}

impl JedecId {
    fn get_device_id(&self) -> u16 {
        ((self.memory_type as u16) << 8) | self.memory_capacity as u16
    }

    fn get_geometry(&self) -> Option<Geometry> {
        let page_size_bytes = 256;

        let (sectors, pages) = match (self.manufacturer, self.get_device_id()) {
            (MFG_MICRON, 0x2015) => (32, 256),
            (MFG_MICRON, 0xBA17) | (MFG_WINBOND, 0x4017) | (MFG_MACRONIX, 0x2017) => (128, 256),
            (MFG_MICRON, 0xBA18) | (MFG_WINBOND, 0x4018) => (256, 256),
            _ => return None
        };

        Some(Geometry {
            page_size_bytes: page_size_bytes,
            pages_per_sector: pages,
            sectors: sectors
        })
    }
}

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(bit_numbering="msb0")]
pub struct Status {
    #[packed_field(bits="0")]
    pub status_register_write_disable: bool,
    #[packed_field(bits="3")]
    pub block_protect_2: bool,
    #[packed_field(bits="4")]
    pub block_protect_1: bool,
    #[packed_field(bits="5")]
    pub block_protect_0: bool,
    #[packed_field(bits="6")]
    pub write_enabled: bool,
    #[packed_field(bits="7")]
    pub write_in_progress: bool
}

registers!(
  chip FlashRegisters {
    register [0x9F; 3] => jedec_id: JedecId,
    register [0x05; 1] => read_status_register: Status,
    register [0x01; 1] => write_status_register: Status
  }
);

pub struct SpiFlashFactory;
impl<S: 'static, B: 'static> DetectableDeviceWithRegistersFactory<SpiFlash<S, B>, S, B> for SpiFlashFactory where S: SystemApi, B: Bus<S> {
    fn get_potential_addresses(&self) -> Vec<BusEnumeration> {
        vec![BusEnumeration::SpiAll]
    }

    fn new(&self, mut register_bus: RegisterBus<S, B>) -> Result<SpiFlash<S, B>, PeripheryError> {
        register_bus.get_bus().get_system_api().get_sleep()?;

        register_bus.set_i2c_style_addressing(false);

        let geometry = {
            let registers = FlashRegisters::new(&register_bus);

            let jedec_id = try!(registers.jedec_id().read());
            let g = try!(jedec_id.get_geometry().ok_or(PeripheryError::ReadError));

            g
        };

        let flash = SpiFlash {
            bus: register_bus,
            geometry: geometry
        };        

        Ok(flash)
    }
}

#[derive(Clone)]
pub struct SpiFlash<S, B> where S: SystemApi, B: Bus<S> {
    bus: RegisterBus<S, B>,
    geometry: Geometry
}

#[derive(PrimitiveEnum_u8, Copy, Clone)]
enum AddressCommand {
    PageProgram = 0x02,
    SectorErase = 0xD8,
    Read = 0x03
}

#[derive(PrimitiveEnum_u8, Copy, Clone)]
enum Command {
    WriteEnable = 0x06,
    WriteDisable = 0x04,
    BulkErase = 0xC7
}

impl<S, B> SpiFlash<S, B> where S: SystemApi, B: Bus<S> {
    pub fn registers<'b>(&'b self) -> FlashRegisters<'b, S, B> {
        FlashRegisters::new(&self.bus)
    }
        
    fn address_cmd(cmd: AddressCommand, address: u32) -> [u8; 4] {
        [
            cmd.to_primitive(),
            ((address >> 16) & 0xFF) as u8, ((address >> 8) & 0xFF) as u8, (address & 0xFF) as u8
        ]
    }

    fn cmd(&self, cmd: Command) -> Result<(), PeripheryError> {
        let cmd = [cmd.to_primitive()];

        match (self.bus.get_bus().get_spi(), self.bus.get_address()) {
            (Ok(spi), ChipOnBus::Spi { chip_number: chip_number} ) => {
                let selected_chip = SpiBusChipSelect::lock(spi, chip_number);
                try!(spi.transmit(&cmd, &mut [0; 1]));                
                drop(selected_chip);
            },
            _ => return Err(PeripheryError::ReadError)
        }

        Ok(())
    }

    fn write_enable(&self) -> Result<(), PeripheryError> {
        self.cmd(Command::WriteEnable)
    }

    fn write_disable(&self) -> Result<(), PeripheryError> {
        self.cmd(Command::WriteDisable)
    }

    fn erase_all(&self) -> Result<(), PeripheryError> {
        try!(self.wait_for_ready(6));
        try!(self.cmd(Command::BulkErase));
        self.wait_for_ready(21000)
    }

    pub fn wait_for_ready(&self, timeout_ms: u32) -> Result<(), PeripheryError> {
        let mut t = timeout_ms as isize;
        loop {
            let status = try!(self.registers().read_status_register().read());
            if status.write_in_progress == false { return Ok(()); }

            if t < 0 { break; }
            self.bus.get_bus().get_system_api().get_sleep()?.sleep_ms(1);
            t -= 1;
        }
        
        Err(PeripheryError::ReadinessTimeout)
    }

    pub fn erase(&self, address: u32) -> Result<(), PeripheryError> {
        try!(self.wait_for_ready(6));        
        try!(self.write_enable());

        let cmd = Self::address_cmd(AddressCommand::SectorErase, address);

        match (self.bus.get_bus().get_spi(), self.bus.get_address()) {
            (Ok(spi), ChipOnBus::Spi { chip_number: chip_number} ) => {
                let selected_chip = SpiBusChipSelect::lock(spi, chip_number);
                try!(spi.transmit(&cmd, &mut [0; 4]));                
                drop(selected_chip);
            },
            _ => return Err(PeripheryError::ReadError)
        }

        self.wait_for_ready(5000)
    }

    pub fn write(&self, address: u32, buf: &[u8]) -> Result<(), PeripheryError> {
        try!(self.wait_for_ready(6));
        try!(self.write_enable());

        let cmd = Self::address_cmd(AddressCommand::PageProgram, address);

        match (self.bus.get_bus().get_spi(), self.bus.get_address()) {
            (Ok(spi), ChipOnBus::Spi { chip_number: chip_number} ) => {
                let selected_chip = SpiBusChipSelect::lock(spi, chip_number);
                try!(spi.transmit(&cmd, &mut [0; 4]));
                let mut dummy_read = vec![0xFF; buf.len()];
                try!(spi.transmit(&buf, &mut dummy_read));
                drop(selected_chip);
            },
            _ => return Err(PeripheryError::ReadError)
        }

        self.wait_for_ready(6)
    }

    pub fn read(&self, address: u32, out: &mut [u8]) -> Result<(), PeripheryError> {        
        //try!(self.wait_for_ready(6));

        let cmd = Self::address_cmd(AddressCommand::Read, address);

        match (self.bus.get_bus().get_spi(), self.bus.get_address()) {
            (Ok(spi), ChipOnBus::Spi { chip_number: chip_number} ) => {
                let selected_chip = SpiBusChipSelect::lock(spi, chip_number);
                try!(spi.transmit(&cmd, &mut [0; 4]));
                let dummy_send = vec![0xFF; out.len()];
                try!(spi.transmit(&dummy_send, out));
                drop(selected_chip);
            },
            _ => return Err(PeripheryError::ReadError)
        }

        Ok(())
    }
}


impl<S, B> Device for SpiFlash<S, B> where S: SystemApi, B: Bus<S> {
    fn get_storage_device(&self) -> Option<&StorageDevice> {
        Some(self)
    }

    fn description(&self) -> Cow<str> {
        format!("SPI Flash at {:?}", self.bus.get_address()).into()
    }

    fn get_cli(&self) -> Option<&DeviceCli> {
        Some(self)
    }

    fn get_registers_cli(&self) -> Option<DeviceBusCli> {
        let mut c = DeviceBusCli::new();
        c.with_registers(self.registers());
        Some(c)
    }

    fn id(&self) -> Cow<str> {
        format!("{}_{}", "spi_flash", self.bus.get_cli_prefix()).into()
    }

    fn init_after_detection(&self) -> Result<bool, PeripheryError> {
        self.registers().write_status_register().write(&Status {
            status_register_write_disable: false,
            block_protect_2: false,
            block_protect_1: false,            
            block_protect_0: false,
            write_enabled: false,
            write_in_progress: false
        });
		self.write_disable();
        Ok(true)
	}
}

impl<S, B> DeviceCli for SpiFlash<S, B> where S: SystemApi, B: Bus<S> {
    fn execute_cli(&self, exec: &mut ::terminal_cli::PrefixedExecutor) {
        use ::periphery_core::terminal_cli::*;

        let prefix = self.id();

        if let Some(mut ctx) = exec.command(&"command/write_enable") {            
            match self.write_enable() {
                Ok(g) => ctx.get_terminal().print_line("Write enabled."),
                Err(e) => ctx.get_terminal().print_line(&format!("Error: {:?}", e))
            }
        }

        if let Some(mut ctx) = exec.command(&"command/write_disable") {            
            match self.write_disable() {
                Ok(g) => ctx.get_terminal().print_line("Write disabled."),
                Err(e) => ctx.get_terminal().print_line(&format!("Error: {:?}", e))
            }
        }

        if let Some(mut ctx) = exec.command(&"command/erase_all") {            
            match self.erase_all() {
                Ok(g) => ctx.get_terminal().print_line("Erase all command sent."),
                Err(e) => ctx.get_terminal().print_line(&format!("Error: {:?}", e))
            }
        }
    }
}

impl<S, B> StorageDevice for SpiFlash<S, B> where S: SystemApi, B: Bus<S> {
    fn get_sector_erase(&self) -> Option<&StorageDeviceSectorErase> {
        Some(self)
    }

    fn get_total_capacity_bytes(&self) -> u64 {
        (self.geometry.page_size_bytes * self.geometry.pages_per_sector * self.geometry.sectors) as u64
    }

    fn get_read_sector_size_bytes(&self) -> u64 {
        self.geometry.page_size_bytes as u64
    }
    fn get_write_sector_size_bytes(&self) -> u64 {
        self.geometry.page_size_bytes as u64
    }

    fn read_sector(&self, address: u64, out: &mut [u8]) -> Result<(), PeripheryError> {
        try!(self.read(address as u32, out));
        Ok(())
    }    

    fn write_sector(&self, address: u64, buf: &[u8]) -> Result<(), PeripheryError> {
        self.write(address as u32, buf)
    }
}

impl<S, B> StorageDeviceSectorErase for SpiFlash<S, B> where S: SystemApi, B: Bus<S> {
    fn get_erase_sector_size_bytes(&self) -> u64 {
        (self.geometry.page_size_bytes * self.geometry.pages_per_sector) as u64
    }

    fn erase_sector(&self, address: u64) -> Result<(), PeripheryError> {
        self.erase(address as u32)
    }
}
