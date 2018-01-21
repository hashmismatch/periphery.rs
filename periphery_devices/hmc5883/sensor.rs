use periphery_core::*;
use periphery_core::prelude::v1::*;

use registers::*;

use packed_struct::prelude::*;

registers!(
  chip Hmc5883Registers {
    register [0x00; 1] => config_a: ConfigurationRegisterA,
    register [0x01; 1] => config_b: ConfigurationRegisterB,
    register [0x02; 1] => mode: ModeRegister,
    
    register [0x03; 6] => data: MagData,
    
    register [0x09; 1] => status: StatusRegister,
  	register [0x0A; 1] => id_a: u8,
    register [0x0B; 1] => id_b: u8,
    register [0x0C; 1] => id_c: u8
  }
);

pub const HMC58X3_X_SELF_TEST_GAUSS: f32 = 1.16;
pub const HMC58X3_Y_SELF_TEST_GAUSS: f32 = 1.16;
pub const HMC58X3_Z_SELF_TEST_GAUSS: f32 = 1.08;


pub type Hmc5883OnI2CBus<B> = Hmc5883<<B as Bus>::SystemApi, <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::Registers>;



#[derive(Clone, Copy)]
pub struct Hmc5883Factory {
    addresses: [I2CAddress; 1]
}

impl Default for Hmc5883Factory {
    fn default() -> Self {
        Hmc5883Factory {
            addresses: [I2CAddress::address_7bit(0x1E)]
        }
    }
}

impl<B> DeviceI2CDetection<Hmc5883OnI2CBus<B>, B, I2CDeviceRegisters<B>> for Hmc5883Factory 
    where B: Bus + 'static,
{
	fn get_addresses(&self) -> &[I2CAddress] {
        &self.addresses
    }

	fn new(args: I2CDeviceRegisters<B>) -> Result<Hmc5883OnI2CBus<B>, PeripheryError> {
        let sensor = Hmc5883 {
            system: args.system_api,
            bus: args.device_bus
        };

        let id_a = sensor.registers().id_a().read()?;
        let id_b = sensor.registers().id_b().read()?;
        let id_c = sensor.registers().id_c().read()?;

        if id_a == 0b01001000 && id_b == 0b00110100 && id_c == 0b00110011 {
            return Ok(sensor)
        }

        Err(PeripheryError::UnsupportedFieldValue)
    }
}

#[derive(Clone)]
pub struct Hmc5883<S, B> where S: SystemApi, B: DeviceRegisterBus {
    system: S,
    bus: B
}

impl<S, B> Hmc5883<S, B> where S: SystemApi, B: DeviceRegisterBus {
    pub fn init_defaults(&self) -> Result<(), PeripheryError> {
        let config_a = ConfigurationRegisterA {
            samples: SamplesAveraged::Samples8,
            data_output_rate: DataOutputRate::Output_15Hz,
            measurement_mode: MeasurementMode::Normal
        };
        self.registers().config_a().write(&config_a)?;

        let config_b = ConfigurationRegisterB {
            gain: Gain::Gain_1090
        };
        self.registers().config_b().write(&config_b)?;

        Ok(())
    }

    pub fn get_magnetic_field_3_raw(&self) -> Result<MagneticField3Raw, PeripheryError> {
        let data = self.registers().data().read()?;

        if data.x == -4096 || data.y == -4096 || data.z == -4096 {
            return Err(PeripheryError::MeasurementOverflow);
        }

        Ok(MagneticField3Raw {
            x: data.x,
            y: data.y,
            z: data.z
        })
    }

    #[inline]
    pub fn registers(&self) -> Hmc5883Registers<B> {
        Hmc5883Registers::new(&self.bus)
    }

    // this really doesn't do much ATM
    /*
    /// The device will be reset to default settings after the calibration.
    pub fn self_calibrate(&self) -> Result<MagneticGains, PeripheryError> {
        let samples_per_bias = 10;
        let mut sum = (0 as i32, 0 as i32, 0 as i32);

        let read_single = || {
            let mode = ModeRegister {
                high_speed_i2c_enabled: false,
                operating_mode: OperatingMode::SingleMeasurement
            };
            try!(self.registers.mode.write(&self.bus, &mode));
            self.system.sleep_ms(60);

            self.get_magnetic_field_3_raw()
        };

        let config_b = ConfigurationRegisterB {
            gain: Gain::Gain_660
        };
        try!(self.registers.config_b.write(&self.bus, &config_b));

        {
            let mut sample = |m: i32| {
                // throw away initial measurements
                try!(read_single());
                try!(read_single());

                for i in 0..samples_per_bias {
                    let data = try!(read_single());
                    sum.0 += (data.x as i32) * m;
                    sum.1 += (data.y as i32) * m;
                    sum.2 += (data.z as i32) * m;
                }
                Ok(())
            };       
            

            let mut config_a = ConfigurationRegisterA {
                samples: SamplesAveraged::Samples8,
                data_output_rate: DataOutputRate::Output_15Hz,
                measurement_mode: MeasurementMode::PositiveBias
            };
            try!(self.registers.config_a.write(&self.bus, &config_a));            

            // Sample with positive bias
            try!(sample(1));

            config_a.measurement_mode = MeasurementMode::NegativeBias;
            try!(self.registers.config_a.write(&self.bus, &config_a));

            // Sample with negative bias
            try!(sample(-1));
        }

        // Reset to defaults
        try!(self.init_defaults());

        let g = (config_b.gain.get_lsb_per_gauss() as f32);
        let samples = 2.0 * (samples_per_bias as f32);
        Ok(MagneticGains {
            x: (g * HMC58X3_X_SELF_TEST_GAUSS * samples) / sum.0 as f32,
            y: (g * HMC58X3_Y_SELF_TEST_GAUSS * samples) / sum.1 as f32,
            z: (g * HMC58X3_Z_SELF_TEST_GAUSS * samples) / sum.2 as f32
        })
    }
    */
}


impl<S, B> Device for Hmc5883<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_magnetic_field_3_sensor(&self) -> Option<&MagneticField3Sensor> {
        Some(self)
    }

    fn description(&self) -> Cow<str> {
        "HMC5883 3 axis compass".into()
    }
    
    fn get_registers_cli(&self) -> Option<DeviceBusCli> {
        let mut c = DeviceBusCli::new();
        c.with_registers(self.registers());
        Some(c)
    }

    fn id(&self) -> Cow<str> {
        "hmc5883".into()
    }

    fn init_after_detection(&self) -> Result<bool, PeripheryError> {
        self.init_defaults()?;
        Ok(true)
    }    
}

impl<S, B> MagneticField3Sensor for Hmc5883<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_magnetic_field_3(&self) -> Result<MagneticField3, PeripheryError> {
        let mode = ModeRegister {
            high_speed_i2c_enabled: false,
            operating_mode: OperatingMode::SingleMeasurement
        };
        self.registers().mode().write(&mode)?;

        self.system.get_sleep()?.sleep_ms(6);        

        let status = self.registers().status().read()?;
        if status.ready == false {
            return Err(PeripheryError::MeasurementNotReady);
        }

        let data = self.get_magnetic_field_3_raw()?;
        let ctrl_b = self.registers().config_b().read()?;

        Ok(data.to_std(&ctrl_b))
    }
}


#[derive(Copy, Clone, Debug, Default)]
pub struct MagneticField3Raw {
    pub x: i16,
    pub y: i16,
    pub z: i16
}

impl MagneticField3Raw {
    pub fn to_std(&self, config_b: &ConfigurationRegisterB) -> MagneticField3 {
        let lsb_per_gauss = config_b.gain.get_lsb_per_gauss() as f32;

        MagneticField3 {
            x: MagneticFieldStrength::from_gauss((self.x as f32) / lsb_per_gauss),
            y: MagneticFieldStrength::from_gauss((self.y as f32) / lsb_per_gauss),
            z: MagneticFieldStrength::from_gauss((self.z as f32) / lsb_per_gauss)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MagneticGains {
    pub x: f32,
    pub y: f32,
    pub z: f32
}

impl Default for MagneticGains {
    fn default() -> Self {
        MagneticGains {
            x: 1.0,
            y: 1.0,
            z: 1.0
        }
    }
}