use periphery_core::*;
use periphery_core::prelude::v1::*;
use periphery_core::terminal_cli::*;

use registers::*;

use packed_struct::*;

use ::gesture_detection::*;

pub type Apds9960OnI2CBus<B> = Apds9960<<B as Bus>::SystemApi, <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::Registers>;

registers!(
  chip Apds9960Registers {
    register [0x80; 1] => enable: EnableRegister,
    register [0x81; 1] => adc_integration_time: AdcIntegrationTime,
    register [0x83; 1] => wait_time: u8,
    register [0x84; 4] => als_interrupt_threshold: AlsInterruptThreshold,
    register [0x89; 1] => proximity_interrupt_low_threshold: u8,
    // is this a typo in the docs?
    register [0x8B; 1] => proximity_interrupt_high_threshold: u8,
    register [0x8C; 1] => persistence: Persistence,
    register [0x8D; 1] => config1: Config1,
    register [0x8E; 1] => proximity_pulse_count: ProximityPulseCount,
    register [0x8F; 1] => control1: ControlRegister1,
    register [0x90; 1] => config2: Config2,



    register [0x92; 1] => id: ChipId,
    register [0x93; 1] => status: Status,

    register [0x94; 2] => channel_clear: LsbU16,
    register [0x96; 2] => channel_red: LsbU16,
    register [0x98; 2] => channel_green: LsbU16,
    register [0x9A; 2] => channel_blue: LsbU16,

    register [0x94; 8] => channels: RGBCData,



    register [0x9C; 1] => proximity_data: u8,
    register [0x9D; 1] => proximity_offset_up_right: u8,
    register [0x9E; 1] => proximity_offset_down_left: u8,

    register [0x9F; 1] => config3: Config3,

    register [0xA0; 1] => gesture_proximity_enter_threshold: u8,
    register [0xA1; 1] => gesture_proximity_exit_threshold: u8,
    register [0xA2; 1] => gesture_config1: GestureConfig1,
    register [0xA3; 1] => gesture_config2: GestureConfig2,
    register [0xA4; 1] => gesture_up_offset: i8,
    register [0xA5; 1] => gesture_down_offset: i8,
    register [0xA7; 1] => gesture_left_offset: i8,
    register [0xA9; 1] => gesture_right_offset: i8,
    register [0xA6; 1] => gesture_pulse_count: GesturePulseCount,

    register [0xAA; 1] => gesture_config3: GestureConfig3,
    register [0xAB; 1] => gesture_config4: GestureConfig4,
    register [0xAE; 1] => gesture_fifo_level: u8,
    register [0xAF; 1] => gesture_status: GestureStatus,

    register [0xE4; 1] => force_interrupt: u8,
    register [0xE5; 1] => proximity_interrupt_clear: u8,
    register [0xE6; 1] => als_interrupt_clear: u8,
    register [0xE7; 1] => clear_all_non_gesture_interrupts: u8,

    register [0xFC; 4] => gesture_fifo: GestureFifo

  }
);


#[derive(Clone, Copy)]
pub struct Apds9960Factory {
    addresses: [I2CAddress; 1]
}

impl Default for Apds9960Factory {
    fn default() -> Self {
        Apds9960Factory {
            addresses: [I2CAddress::address_7bit(0x39)]
        }
    }
}

impl<B> DeviceI2CDetection<Apds9960OnI2CBus<B>, B, I2CDeviceRegisters<B>> for Apds9960Factory 
    where B: Bus + 'static,
{
	fn get_addresses(&self) -> &[I2CAddress] {
        &self.addresses
    }

	fn new(args: I2CDeviceRegisters<B>) -> Result<Apds9960OnI2CBus<B>, PeripheryError> {
        let sensor = Apds9960 {
            system: args.system_api,
            bus: args.device_bus
        };

        let id = sensor.registers().id().read()?;

        Ok(sensor)
    }
}


#[derive(Debug, Clone)]
pub enum Apds9960Error {
    AmbientLightSensorValueNotValid
}

impl From<Apds9960Error> for PeripheryError {
	fn from(err: Apds9960Error) -> Self {		
		PeripheryError::SensorError {
            sensor: "apds9960".into(),
            error: format!("{:?}", err).into()
        }
	}
}


#[derive(Clone)]
pub struct Apds9960<S, B> where S: SystemApi, B: DeviceRegisterBus {
    system: S,
    bus: B,
}

impl<S, B> Apds9960<S, B> where S: SystemApi, B: DeviceRegisterBus {
    #[inline]
    pub fn registers<'a>(&'a self) -> Apds9960Registers<'a, B> {
        Apds9960Registers::new(&self.bus)
    }

    pub fn init(&self) -> Result<(), PeripheryError> {
        let r = self.registers();

        let mut enable_register = EnableRegister {
            gesture_enable: false,
            proximity_interrupt_enable: false,
            als_interrupt_enable: false,
            wait_enable: false,
            proximity_detect_enable: false,
            als_enable: false,
            power_on: false
        };

        r.enable().write(&enable_register)?;

        r.adc_integration_time().write(&AdcIntegrationTime::new_from_cycles(10))?; 
        r.wait_time().write(&246)?; // 27ms
        r.proximity_pulse_count().write(&ProximityPulseCount {
            proximity_pulse_length: 2.into(),
            proximity_pulse_count: 8.into()
        })?; // 16us, 8 pulses
        r.proximity_offset_up_right().write(&0)?;
        r.proximity_offset_down_left().write(&0)?;

        r.config1().write(&Config1 {
            wait_long: false
        })?;
        r.control1().write(&ControlRegister1 {
            led_drive_strength: 0.into(),
            proximity_gain_control: ProximityGain::Times2,
            als_and_color_gain: AlsGain::Times4
        })?;

        
        r.proximity_interrupt_low_threshold().write(&0)?;
        r.proximity_interrupt_low_threshold().write(&50)?;
        r.persistence().write(&Persistence {
            proximity_interrupt_persistence: 1.into(),
            als_interrupt_persistence: 1.into(),
        })?;

        r.config2().write(&Config2 {
            proximity_saturation_interrupt_enable: false,
            clear_photodiode_interrupt_enable: false,
            led_boost: 0.into(),
            reserved_true: true
        })?;

        r.config3().write(&Config3 {
            proximity_gain_compensation_enable: false,
            sleep_after_interrupt: false,
            proximity_mask_up: false,
            proximity_mask_down: false,
            proximity_mask_left: false,
            proximity_mask_right: false
        })?;

        r.gesture_proximity_enter_threshold().write(&40)?;
        r.gesture_proximity_exit_threshold().write(&30)?;

        r.gesture_config1().write(&GestureConfig1 {
            gesture_fifo_threshold: 1.into(),
            gesture_exit_mask: 0.into(),
            gesture_exit_persistence: 0.into()
        })?;

        r.gesture_config2().write(&GestureConfig2 {
            gesture_gain: GestureGain::Times1,
            gesture_led_drive_strength: GestureLedDrive::Current100mA,
            gesture_wait_time: 1.into()
        })?;

        r.gesture_up_offset().write(&0)?;
        r.gesture_down_offset().write(&0)?;
        r.gesture_left_offset().write(&0)?;
        r.gesture_right_offset().write(&0)?;


        r.gesture_pulse_count().write(&GesturePulseCount {
            gesture_pulse_length: 3.into(),
            number_of_gesture_pulses: 9.into()
        })?;

        r.gesture_config3().write(&GestureConfig3 {
            gesture_dimension: 0.into()
        })?;

        

        enable_register.als_enable = true;
        enable_register.power_on = true;

        r.enable().write(&enable_register)?;

        Ok(())
    }

    pub fn enable_gestures(&self) -> Result<(), PeripheryError> {
        let r = self.registers();

        // always on, do not exit gesture mode
        r.gesture_proximity_enter_threshold().write(&0)?;
        r.gesture_proximity_exit_threshold().write(&0)?;

        r.wait_time().write(&0xFF)?; // 2.78ms
        r.proximity_pulse_count().write(&ProximityPulseCount {
            proximity_pulse_length: 1.into(),
            proximity_pulse_count: 9.into()
        })?; // 16us, 8 pulses
        r.gesture_config2().write(&GestureConfig2 {
            gesture_gain: GestureGain::Times2,
            gesture_led_drive_strength: GestureLedDrive::Current100mA,
            gesture_wait_time: 0.into()
        })?;
        r.gesture_config4().write(&GestureConfig4 {
            gesture_fifo_clear: true,
            gesture_interrupt_enable: false,
            gesture_mode: true
        })?;
        r.enable().modify(|e| {
            e.power_on = true;
            e.wait_enable = true;
            e.proximity_detect_enable = true;
            e.gesture_enable = true;
        })?;

        Ok(())
    }

    pub fn is_gesture_available(&self) -> Result<bool, PeripheryError> {
        let status = self.registers().gesture_status().read()?;
        Ok(status.gesture_fifo_data_valid)
    }

    pub fn get_gestures(&self) -> Result<Vec<GestureFifo>, PeripheryError> {
        use packed_struct::*;

        let mut ret = vec![];
        let w = GestureFifo::packed_bytes();

        loop {
            let samples_count = self.registers().gesture_fifo_level().read()? as usize;
            if samples_count <= 0 { break; }            
            
            let mut buffer = vec![0; samples_count * w];
            let raw_data = self.bus.read_from_register(0xFC, &mut buffer)?;
            for i in 0..samples_count {
                let b = &buffer[(i * w)..((i+1) * w)];
                let gesture = GestureFifo::unpack_from_slice(&b)?;
                ret.push(gesture);
            }
        }

        Ok(ret)
    }

    pub fn get_ambient_light_settings(&self) -> Result<AmbientLightSettings, PeripheryError> {
        let s = AmbientLightSettings {
            adc_integration_time: self.registers().adc_integration_time().read()?,
            als_and_color_gain: self.registers().control1().read()?.als_and_color_gain
        };

        Ok(s)
    }

    pub fn get_ambient_light_raw(&self) -> Result<RGBCData, PeripheryError> {
        let status = self.registers().status().read()?;
        if status.als_valid == false {
            return Err(Apds9960Error::AmbientLightSensorValueNotValid.into());
        }

        Ok(self.registers().channels().read()?)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AmbientLightSettings {
    pub adc_integration_time: AdcIntegrationTime,
    pub als_and_color_gain: AlsGain
}

impl AmbientLightSettings {
    pub fn sensor_to_physical(&self, ambient_light: RGBCData) -> Result<Illuminance, PeripheryError> {
        let glass_attenuation_factor = 0.49;
        let device_factor = 52.0;

        let als_integration_time_ms = self.adc_integration_time.get_integration_time_ms();
        let als_gain = self.als_and_color_gain.get_gain_factor() as f32;

        let lux_per_count = (glass_attenuation_factor * device_factor) / (als_integration_time_ms * als_gain);

        // check for saturated channels
        let max_channel_value = self.adc_integration_time.get_max_value();
        if ambient_light.red >= max_channel_value ||
           ambient_light.green >= max_channel_value ||
           ambient_light.blue >= max_channel_value ||
           ambient_light.clear >= max_channel_value
        {
            return Err(PeripheryError::MeasurementOverflow);
        }

        let ir = (ambient_light.red as i32 + ambient_light.green as i32 + ambient_light.blue as i32 - ambient_light.clear as i32) / 2;

        let red = ambient_light.red as i32 - ir;
        let green = ambient_light.green as i32 - ir;
        let blue = ambient_light.blue as i32 - ir;

        // todo: correction coefficients?
        let ir_adjusted_count = red + green + blue;

        let lux = ir_adjusted_count as f32 * lux_per_count;

        // as of right now, an unknown correction factor
        let lux = lux * 5.5;

        let i = Illuminance::from_lux(lux);
        Ok(i)
    }
}

impl<S, B> AmbientLightSensor for Apds9960<S, B> where S: SystemApi, B: DeviceRegisterBus {
	fn get_ambient_light(&self) -> Result<Illuminance, PeripheryError> {
        let ambient_light = self.get_ambient_light_raw()?;
        let settings = self.get_ambient_light_settings()?;

        settings.sensor_to_physical(ambient_light)
    }
}

impl<S: 'static, B: 'static> Device for Apds9960<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn description(&self) -> Cow<str> {
        "APDS-9960 Digital Proximity, Ambient Light, RGB and Gesture Sensor".into()
    }

    fn get_registers_cli(&self) -> Option<DeviceBusCli> {
        let mut c = DeviceBusCli::new();
        c.with_registers(self.registers());
        Some(c)
    }

    fn id(&self) -> Cow<str> {
        "apds9960".into()
    }

    fn get_ambient_light_sensor(&self) -> Option<&AmbientLightSensor> {
		Some(self)
	}

    fn init_after_detection(&self) -> Result<bool, PeripheryError> {
        self.init()?;
		Ok(true)
	}

	fn get_cli(&self) -> Option<&DeviceCli> {
		Some(self)
	}

    fn get_data_streams(&self) -> Option<&DataStreams> {
		Some(self)
	}
}

impl<S: 'static, B: 'static> DeviceCli for Apds9960<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn execute_cli(&self, exec: &mut PrefixedExecutor) {
        if let Some(mut ctx) = exec.command(&"init") {
            self.init();
        }


        if let Some(mut ctx) = exec.command(&"gestures/enable") {
            self.enable_gestures();
        }

        if let Some(mut ctx) = exec.command(&"gestures/is_available") {
            if let Ok(is_available) = self.is_gesture_available() {
                write!(ctx.get_terminal(), "Is gesture data available: {}\r\n", is_available);
            }
        }

        if let Some(mut ctx) = exec.command(&"gestures/raw_gestures") {
            if let Ok(gestures) = self.get_gestures() {
                write!(ctx.get_terminal(), "Gestures FIFO ({})", gestures.len());
                for g in &gestures {
                    write!(ctx.get_terminal(), "{:?}\r\n", g);
                }
            }
        }
    }
}

fn datastream_2_info() -> DataStream {
    DataStream {
        id: DataStreamId(2),
        cli_id: "gestures_raw_fixed_gain".into(),
        description: "Dumps raw gesture sensor data, fixed gain".into(),
        poll_every_ms: 10,
        labels: vec![
            "up".into(),
            "down".into(),
            "left".into(),
            "right".into()
        ]
    }
}

fn datastream_10_info() -> DataStream {
    DataStream {
        id: DataStreamId(10),
        cli_id: "gestures".into(),
        description: "Detects gestures".into(),
        poll_every_ms: 10,
        labels: vec![
            "GestureStep".into()
        ]
    }
}

impl<S: 'static, B: 'static> DataStreams for Apds9960<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_stream_infos(&self) -> Vec<DataStream> {
        vec![datastream_2_info(), datastream_10_info()]
    }

    fn get_poller(&self, stream: DataStreamId) -> Result<Box<DataStreamPoller + Send + Sync>, PeripheryError> {
        /*
        match stream {            
            DataStreamId(2) => {
                let device = self.clone();
                device.enable_gestures()?;

                let p = RawGesturePoller {
                    info: datastream_2_info(),
                    device: device
                };
                Ok(Box::new(p))
            },
            DataStreamId(10) => {
                let device = self.clone();
                device.enable_gestures()?;

                let p = GestureDetectorPoller {
                    device: device,
                    max_measurements: 500,
                    ms: 0,
                    buffer: vec![]
                };
                Ok(Box::new(p))
            },
            _ => {
                Err(PeripheryError::NotImplemented)
            }
        }
        */
        Err(PeripheryError::NotImplemented)
    }
}

pub struct RawGesturePoller<S: 'static, B: 'static> where S: SystemApi, B: DeviceRegisterBus {
    device: Apds9960<S, B>,
    info: DataStream
}

impl<S: 'static, B: 'static> DataStreamPoller for RawGesturePoller<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_info(&self) -> DataStream {
        self.info.clone()
    }

	fn poll(&mut self) -> Result<Vec<DataStreamPolled>, PeripheryError> {
        let mut ret = vec![];
        let mut raw = vec![];

        let mult = 1.0;

        let gestures = self.device.get_gestures()?;
        for gesture in gestures {
            raw.push(gesture);
            
            let polled = DataStreamPolled::F32 {
                data: vec![
                    (gesture.up as f32) * mult,
                    (gesture.down as f32) * mult,
                    (gesture.left as f32) * mult,
                    (gesture.right as f32) * mult
                ]
            };            

            ret.push(polled);
        }

        return Ok(ret);
    }
}

pub struct GestureDetectorPoller<S, B> where S: SystemApi, B: DeviceRegisterBus {
    device: Apds9960<S, B>,
    max_measurements: usize,
    buffer: Vec<GestureSensorPoint>,
    ms: u64
}

impl<S: 'static, B: 'static> DataStreamPoller for GestureDetectorPoller<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_info(&self) -> DataStream {
        datastream_2_info()
    }

	fn poll(&mut self) -> Result<Vec<DataStreamPolled>, PeripheryError> {
        self.ms += 1;
        
        while self.device.registers().gesture_fifo_level().read()? > 0 {
            let gesture = self.device.registers().gesture_fifo().read()?;

            let data = GestureSensorPoint {
                milliseconds: self.ms as f32,
                up: (gesture.up as f32),
                down: (gesture.down as f32),
                left: (gesture.left as f32),
                right: (gesture.right as f32)
            };
            
            self.buffer.push(data);
        }

        let mut ret = vec![];

        let mut detector = GestureDetector::new();
        let gestures = detector.detect(&self.buffer);

        for gesture in &gestures {
            let ev = DataStreamPolled::Strings {
                data: vec![format!("{:?}", gesture).into()]
            };
            ret.push(ev);
        }

        if gestures.len() > 0 {
            self.buffer.clear();
        }

        while self.buffer.len() > self.max_measurements {
            self.buffer.remove(0);
        }

        return Ok(ret);
    }
}
