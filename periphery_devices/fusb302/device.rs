use periphery_core::*;
use periphery_core::prelude::v1::*;
use periphery_core::terminal_cli::*;

use registers::*;

use packed_struct::*;

pub const FIFO_ADDRESS: u8 = 0x43; 

registers!(
  chip Fusb302Registers {
    register [0x01; 1] => device_id: DeviceId,
    register [0x02; 1] => switches0: Switches0,
    register [0x03; 1] => switches1: Switches1,
    register [0x04; 1] => measure: Measure,
    register [0x05; 1] => slice: Slice,
    register [0x06; 1] => control0: Control0,
    register [0x07; 1] => control1: Control1,
    register [0x08; 1] => control2: Control2,
    register [0x09; 1] => control3: Control3,
    register [0x0A; 1] => mask: Mask,
    register [0x0B; 1] => power: Power,
    register [0x0C; 1] => reset: Reset,
    register [0x0D; 1] => ocp_reg: OcpReg,
    register [0x0E; 1] => mask_a: MaskA,
    register [0x0F; 1] => mask_b: MaskB,
    register [0x3C; 1] => status0a: Status0A,
    register [0x3D; 1] => status1a: Status1A,
    register [0x3E; 1] => interrupt_a: InterruptA,
    register [0x3F; 1] => interrupt_b: InterruptB,    
    register [0x40; 1] => status0: Status0,
    register [0x41; 1] => status1: Status1,
    register [0x42; 1] => interrupt: Interrupt,
    register [FIFO_ADDRESS; 1] => fifo: u8
  }
);

pub type Fusb302OnI2CBus<B> = Fusb302<
        <B as Bus>::SystemApi,
        <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::Registers,
        <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::DataTransfer,
    >;

#[derive(Clone, Copy)]
pub struct Fusb302Factory {
    addresses: [I2CAddress; 4]
}

impl Default for Fusb302Factory {
    fn default() -> Self {
        Fusb302Factory {
            addresses: [
                I2CAddress::address_7bit(0x22),
                I2CAddress::address_7bit(0x23),
                I2CAddress::address_7bit(0x24),
                I2CAddress::address_7bit(0x25)
            ]
        }
    }
}

impl<B> DeviceI2CDetection<Fusb302OnI2CBus<B>, B, I2CDeviceAll<B>> for Fusb302Factory 
    where B: Bus + 'static,
{
	fn get_addresses(&self) -> &[I2CAddress] {
        &self.addresses
    }

	fn new(args: I2CDeviceAll<B>) -> Result<Fusb302OnI2CBus<B>, PeripheryError> {
        let device = Fusb302 {
            system: args.system_api,
            bus: args.device_registers,
            data: args.device_data
        };

        match device.registers().device_id().read() {
            Ok(device_id) => {
                if *device_id.version == 9 && *device_id.revision >= 1 {
                    Ok(device)
                } else {
                    Err(PeripheryError::UnsupportedFieldValue)
                }
            },
            Err(e) => Err(e)
        }
    }
}

#[derive(Clone)]
pub struct Fusb302<S, B, D: > where S: SystemApi, B: DeviceRegisterBus, D: DeviceDataTransfer {
    system: S,
    bus: B,
    data: D
}

impl<S, B, D> Fusb302<S, B, D> where S: SystemApi, B: DeviceRegisterBus, D: DeviceDataTransfer {
    #[inline]
    pub fn registers<'a>(&'a self) -> Fusb302Registers<'a, B> {
        Fusb302Registers::new(&self.bus)
    }

    pub fn init(&self) -> Result<bool, PeripheryError> {

        self.registers().reset().write(&Reset {
            pd_reset: false,
            sw_reset: true
        })?;

        self.registers().control3().write(&Control3 {
            auto_retry: true,
            auto_softreset: true,
            auto_hardreset: true,
            n_retries: 3.into(),
            ..Default::default()
        })?;

        self.set_polarity(false, Polarity::TransmitCC1)?;

        // on-chip automatic CRC packet handling
        {
            self.registers().switches1().modify(|s| {
                s.auto_crc = true;            
            })?;
            self.registers().control0().modify(|c| {
                c.auto_pre = false;
            })?;
        }

        self.registers().power().write(&Power {
            pwr: 0xF.into()
        })?;

        Ok(true)
    }

    pub fn pd_reset(&self) -> Result<(), PeripheryError> {
        self.registers().reset().write(&Reset {
            pd_reset: true,
            sw_reset: false
        })?;
        Ok(())
    }

    pub fn set_polarity(&self, vconn_enabled: bool, polarity: Polarity) -> Result<(), PeripheryError> {
        self.registers().switches0().modify(|s| {
            s.vcon_cc1 = false;
            s.vcon_cc2 = false;

            if vconn_enabled {
                match polarity {
                    Polarity::TransmitCC2 => { s.vcon_cc1 = true; },
                    Polarity::TransmitCC1 => { s.vcon_cc2 = true; }
                }
            }

            s.meas_cc1 = false;
            s.meas_cc2 = false;

            match polarity {
                Polarity::TransmitCC2 => { s.meas_cc2 = true; },
                Polarity::TransmitCC1 => { s.meas_cc1 = true; }
            }
        })?;

        self.registers().switches1().modify(|s| {
            s.txcc1 = false;
            s.txcc2 = false;

            match polarity {
                Polarity::TransmitCC2 => { s.txcc2 = true; },
                Polarity::TransmitCC1 => { s.txcc1 = true; }
            }

            s.auto_crc = true;
        })?;

        Ok(())
    }

    pub fn detect_cc_pin_sink(&self) -> Result<CcVoltageStatus, PeripheryError> {
        let original_state = self.registers().switches0().read()?;

        let cc1_status = {
            self.registers().switches0().modify(|s| {
                s.meas_cc1 = true;
                s.meas_cc2 = false;
            })?;

            self.system.sleep_ms(10);

            self.registers().status0().read()?
        };

        let cc2_status = {
            self.registers().switches0().modify(|s| {
                s.meas_cc1 = false;
                s.meas_cc2 = true;
            })?;

            self.system.sleep_ms(10);

            self.registers().status0().read()?
        };

        self.registers().switches0().write(&original_state)?;

        Ok(CcVoltageStatus {
            cc1: convert_bc_lvl(cc1_status, false),
            cc2: convert_bc_lvl(cc2_status, false)
        })
    }

    pub fn fifo_read_all(&self) -> Result<Vec<u8>, PeripheryError> {
        let mut r = vec![];

        loop {
            let status = self.registers().status1().read()?;
            if status.rx_empty { break; }

            r.push(self.registers().fifo().read()?);
        }

        Ok(r)
    }

    pub fn fifo_send_message(&self, data: &[u8]) -> Result<(), PeripheryError> {
        let mut fifo_buffer = vec![];

        let FUSB302_TKN_TXON = 0xA1;
        let FUSB302_TKN_SYNC1 = 0x12;
        let FUSB302_TKN_SYNC2 = 0x13;
        let FUSB302_TKN_SYNC3 = 0x1B;
        let FUSB302_TKN_RST1 = 0x15;
        let FUSB302_TKN_RST2 = 0x16;
        let FUSB302_TKN_PACKSYM = 0x80;
        let FUSB302_TKN_JAMCRC = 0xFF;
        let FUSB302_TKN_EOP = 0x14;
        let FUSB302_TKN_TXOFF = 0xFE;

        fifo_buffer.push(FUSB302_TKN_SYNC1);
        fifo_buffer.push(FUSB302_TKN_SYNC1);
        fifo_buffer.push(FUSB302_TKN_SYNC1);
        fifo_buffer.push(FUSB302_TKN_SYNC2);
        
        fifo_buffer.push(FUSB302_TKN_PACKSYM | ((data.len()) as u8) & 0x1F);        
        fifo_buffer.extend_from_slice(data);
        
        fifo_buffer.push(FUSB302_TKN_JAMCRC);
        fifo_buffer.push(FUSB302_TKN_EOP);
        fifo_buffer.push(FUSB302_TKN_TXOFF);
        
        self.registers().control0().modify(|c| {
            c.tx_flush = true;
        })?;

        self.bus.write_to_register(FIFO_ADDRESS, &fifo_buffer)?;

        self.registers().control0().modify(|c| {
            c.tx_start = true;
        })?;
        
        Ok(())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CcVoltageStatus {
    pub cc1: TypeC_CC_VoltageStatus,
    pub cc2: TypeC_CC_VoltageStatus
}

impl CcVoltageStatus {
    pub fn get_polarity(&self) -> Option<Polarity> {
        if self.cc1 == self.cc2 {
            None
        } else if self.cc1 > self.cc2 {
            Some(Polarity::TransmitCC1)
        } else {
            Some(Polarity::TransmitCC2)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Polarity {
    TransmitCC1,
    TransmitCC2
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum TypeC_CC_VoltageStatus {
    Open,
    Ra,
    Rd,
    SinkDefault,
    Sink15,
    Sink30
}

fn convert_bc_lvl(status: Status0, pulling_up: bool) -> TypeC_CC_VoltageStatus {
    if pulling_up {
        if *status.bc_lvl == 0 {
            TypeC_CC_VoltageStatus::Ra
        } else if *status.bc_lvl < 0x3 {
            TypeC_CC_VoltageStatus::Rd
        } else {
            TypeC_CC_VoltageStatus::Open
        }
    } else {
        match *status.bc_lvl {
            0x1 => TypeC_CC_VoltageStatus::SinkDefault,
            0x2 => TypeC_CC_VoltageStatus::Sink15,
            0x3 => TypeC_CC_VoltageStatus::Sink30,
            _ => TypeC_CC_VoltageStatus::Open
        }
    }
}

impl<S, B, D> Device for Fusb302<S, B, D> where S: SystemApi, B: DeviceRegisterBus, D: DeviceDataTransfer {
    fn description(&self) -> Cow<str> {
        "FUSB302 Programmable USB Type-C Controller with PD".into()
    }

    fn get_registers_cli(&self) -> Option<DeviceBusCli> {
        let mut c = DeviceBusCli::new();
        c.with_registers(self.registers());
        Some(c)
    }

    fn id(&self) -> Cow<str> {
        "fusb302".into()
    }
}
