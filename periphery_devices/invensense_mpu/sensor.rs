use periphery_core::*;
use periphery_core::prelude::v1::*;
use periphery_core::terminal_cli::*;

use registers::*;

use packed_struct::*;


registers!(
  chip MpuRegisters {
    register [0x19; 1] => sample_rate_divider: u8,
    register [0x1A; 1] => config: ConfigRegister,
    register [0x1B; 1] => gyro_config: GyroConfig,
    register [0x1C; 1] => accel_config: AccelConfig,

    register [0x3B; 6] => accel: AcceleratorData,
    register [0x41; 2] => temp: TemperatureData,
    register [0x3B; 14] => acc_temp_gyro: AccTempGyroData,
    register [0x43; 6] => gyro: GyroscopeData,

    register [0x68; 1] => signal_reset: SignalPathReset,
    register [0x6b; 1] => power1: PowerManagement1,
    register [0x6c; 1] => power2: PowerManagement2,
    register [0x37; 1] => int_pin_cfg: InterruptPinConfig,
    register [0x38; 1] => interrupt_enable: InterruptEnable,
    register [0x3A; 1] => interrupt_status: InterruptStatus,
    register [0x6A; 1] => usr_control: UserControl,

  	register [0x75; 1] => who_am_i: u8
  }
);


pub type InvensenseMpuOnI2CBus<B> = InvensenseMpu<<B as Bus>::SystemApi, <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::Registers>;

#[derive(Clone, Copy)]
pub struct InvensenseMpuFactory {
    addresses: [I2CAddress; 2]
}

impl Default for InvensenseMpuFactory {
    fn default() -> Self {
        InvensenseMpuFactory {
            addresses: [
                I2CAddress::address_7bit(0x68),
                I2CAddress::address_7bit(0x69)
            ]
        }
    }
}

impl<B> DeviceI2CDetection<InvensenseMpuOnI2CBus<B>, B, I2CDeviceRegisters<B>> for InvensenseMpuFactory 
    where B: Bus + 'static,
{
	fn get_addresses(&self) -> &[I2CAddress] {
        &self.addresses
    }

	fn new(args: I2CDeviceRegisters<B>) -> Result<InvensenseMpuOnI2CBus<B>, PeripheryError> {        
        let chip = {
            let registers = MpuRegisters::new(&args.device_bus);

            let id = registers.who_am_i().read()?;
            let chip = InvenseMpuChip::from_who_am_i(id).ok_or(PeripheryError::UnsupportedDevice)?;
            chip
        };
        
        let sensor = InvensenseMpu {
            system: args.system_api,
            bus: args.device_bus,
            chip: chip
        };
        
        Ok(sensor)        
    }
}

#[derive(Clone)]
pub struct InvensenseMpu<S, B> {
    system: S,
    bus: B,
    chip: InvenseMpuChip
}

impl<S, B> InvensenseMpu<S, B> where S: SystemApi, B: DeviceRegisterBus {
    pub fn registers<'b>(&'b self) -> MpuRegisters<'b, B> {
        MpuRegisters::new(&self.bus)
    }
    
    pub fn reset_device(&self) -> Result<(), PeripheryError> {
        self.registers().power1().write(&PowerManagement1 {
            device_reset: true,
            sleep: false,
            cycle: false,
            temperature_disabled: false,
            clock_source: ClockSource::Internal
        })?;
        self.system.sleep_ms(150);

        self.registers().signal_reset().write(&SignalPathReset {
            gyro_reset: true,
            accel_reset: true,
            temperature_reset: true
        })?;
        self.system.sleep_ms(150);

        self.registers().power1().write(&PowerManagement1 {
            device_reset: false,
            sleep: false,
            cycle: false,
            temperature_disabled: false,
            clock_source: ClockSource::Internal
        })?;
        self.system.sleep_ms(150);
        
        Ok(())
    }

    pub fn init_defaults(&self) -> Result<(), PeripheryError> {
        self.reset_device()?;

        self.registers().power2().write(&PowerManagement2 {
            lp_wake_ctrl: WakeCtrl::Freq_1_25Hz,
            stby_xa: false,
            stby_ya: false,
            stby_za: false,
            stby_xg: false,
            stby_yg: false,
            stby_zg: false
        })?;
        self.system.sleep_ms(10);

        self.registers().gyro_config().write(&GyroConfig {
            x_axis_self_test_enabled: false,
            y_axis_self_test_enabled: false,
            z_axis_self_test_enabled: false,
            scale: GyroFullScale::Scale_2000
        })?;
        self.system.sleep_ms(10);

        self.registers().accel_config().write(&AccelConfig {
            x_axis_self_test_enabled: false,
            y_axis_self_test_enabled: false,
            z_axis_self_test_enabled: false,
            scale: AccelerometerFullScale::Scale_8g
        })?;
        self.system.sleep_ms(10);

        let config = ConfigRegister {
            ext_sync_set: ExtSync::InputDisabled,
            dlpf_cfg: DigitalLowPassFilter::Filter2
        };
        self.registers().config().write(&config)?;
        self.system.sleep_ms(10);

        self.set_sensor_sampling_rate(&config, 500)?;
        self.system.sleep_ms(10);

        Ok(())
    }

    pub fn get_acceleration_3_raw(&self) -> Result<Acceleration3Raw, PeripheryError> {
        let data = self.registers().accel().read()?;

        Ok(Acceleration3Raw { x: data.x, y: data.y, z: data.z })
    }

    pub fn get_angular_speed_3_raw(&self) -> Result<AngularSpeed3Raw, PeripheryError> {
        let data = self.registers().gyro().read()?;
        
        Ok(AngularSpeed3Raw { x: data.x, y: data.y, z: data.z })
    }

    pub fn get_all_data_raw(&self) -> Result<(Acceleration3Raw, TemperatureRaw, AngularSpeed3Raw), PeripheryError> {
        let data = self.registers().acc_temp_gyro().read()?;

        Ok((
            Acceleration3Raw { x: data.acceleration.x, y: data.acceleration.y, z: data.acceleration.z },
            TemperatureRaw { temp: data.temperature.temperature },
            AngularSpeed3Raw { x: data.gyroscope.x, y: data.gyroscope.y, z: data.gyroscope.z }
        ))
    }

    pub fn set_sensor_sampling_rate(&self, config: &ConfigRegister, rate_hz: u32) -> Result<(), PeripheryError> {        
        let r = (config.dlpf_cfg.get_gyroscope_output_rate_hz() / rate_hz) as u8 - 1;
        self.registers().sample_rate_divider().write(&r)?;
        Ok(())
    }

    pub fn get_sensor_sampling_rate_hz(&self, config: &ConfigRegister) -> Result<u32, PeripheryError> {
        let s = self.registers().sample_rate_divider().read()?;
        Ok(config.dlpf_cfg.get_gyroscope_output_rate_hz() / (1 + s as u32))
    }
}


impl<S, B> Device for InvensenseMpu<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn description(&self) -> Cow<str> {
        format!("Invensense {:?}", self.chip).into()
    }

    fn get_acceleration_3_sensor(&self) -> Option<&Acceleration3Sensor> {
        Some(self)
    }

    fn get_ambient_temperature_sensor(&self) -> Option<&AmbientTemperatureSensor> {
        Some(self)
    }

    fn get_angular_speed_3_sensor(&self) -> Option<&AngularSpeed3Sensor> {
        Some(self)
    }
    
    fn id(&self) -> Cow<str> {
        "mpu".into()
    }

    fn get_registers_cli(&self) -> Option<DeviceBusCli> {
        let mut c = DeviceBusCli::new();
        c.with_registers(self.registers());
        Some(c)
    }

    fn init_after_detection(&self) -> Result<bool, PeripheryError> {
        try!(self.init_defaults());
        Ok(true)
    }    
}

impl<S, B> Acceleration3Sensor for InvensenseMpu<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_acceleration_3(&self) -> Result<Acceleration3, PeripheryError> {
        let raw = try!(self.get_acceleration_3_raw());
        let accel_config = try!(self.registers().accel_config().read());

        Ok(raw.to_std_units(&accel_config))
    }
}

impl<S, B> AngularSpeed3Sensor for InvensenseMpu<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_angular_speed_3(&self) -> Result<AngularSpeed3, PeripheryError> {
        let raw = try!(self.get_angular_speed_3_raw());
        let gyro_config = try!(self.registers().gyro_config().read());

        Ok(raw.to_std_units(&gyro_config))
    }
}

impl<S, B> AmbientTemperatureSensor for InvensenseMpu<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_ambient_temperature(&self) -> Result<AmbientTemperature, PeripheryError> {
        let t = try!(self.registers().temp().read()).temperature;
        let t = ((t as f32)/340.0) + 36.53;

        Ok(AmbientTemperature::from_temperature(Temperature::from_degrees_celsius(t)))
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum InvenseMpuChip {
    Mpu60x0,
    Mpu65x0,
    Icm20601,
    Icm20602
}

impl InvenseMpuChip {
    pub fn from_who_am_i(id: u8) -> Option<InvenseMpuChip> {
        match id {
            0x68 => Some(InvenseMpuChip::Mpu60x0),
            0x70 => Some(InvenseMpuChip::Mpu65x0),
            0xAC => Some(InvenseMpuChip::Icm20601),
            0x12 => Some(InvenseMpuChip::Icm20602),
            _ => None
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct TemperatureRaw {
    pub temp: i16
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Acceleration3Raw {
    pub x: i16,
    pub y: i16,
    pub z: i16
}

impl Acceleration3Raw {
    pub fn to_std_units(&self, accel_config: &AccelConfig) -> Acceleration3 {
        let scale = 1.0 / accel_config.scale.get_lsb_per_g() as f32;

        Acceleration3::new(
            GForce::from_g_force(scale * self.x as f32),
            GForce::from_g_force(scale * self.y as f32),
            GForce::from_g_force(scale * self.z as f32),
        )
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct AngularSpeed3Raw {
    pub x: i16,
    pub y: i16,
    pub z: i16
}

impl AngularSpeed3Raw {
    pub fn to_std_units(&self, gyro_config: &GyroConfig) -> AngularSpeed3 {
        let scale = 1.0 / gyro_config.scale.get_lsb_per_deg_per_s() as f32;

        AngularSpeed3 {
            x: AngularSpeed::from_degrees_per_second(scale * self.x as f32),
            y: AngularSpeed::from_degrees_per_second(scale * self.y as f32),
            z: AngularSpeed::from_degrees_per_second(scale * self.z as f32)
        }
    }
}
