use periphery_core::*;
use periphery_core::prelude::v1::*;

use packed_struct::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PrimitiveEnum_u8)]
pub enum Oversampling {
    UltraLowPower = 0,
    LowPower = 1,
    Standard = 2,
    HighRes = 3,
    UltraHighRes = 4
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DataRequest {
    Temperature,
    Pressure
}

impl Oversampling {
    pub fn get_conversion_ms_delay(&self) -> u32 {
        match *self {
            Oversampling::UltraLowPower => 1,
            Oversampling::LowPower => 3,
            Oversampling::Standard => 4,
            Oversampling::HighRes => 6,
            Oversampling::UltraHighRes => 10
        }
    }
}

type MsbU16 = MsbInteger<u16, packed_bits::Bits16, Integer<u16, packed_bits::Bits16>>;
type MsbU24 = MsbInteger<u32, packed_bits::Bits24, Integer<u32, packed_bits::Bits24>>;

registers!(
  chip Ms5611Registers {
    register [0x00; 3] => adc: MsbU24,

    register [0x1E; 1] => reset: u8,

  	register [0xA0; 2] => factory_data: MsbU16,
    register [0xA2; 2] => coeff_1: MsbU16,
    register [0xA4; 2] => coeff_2: MsbU16,
    register [0xA6; 2] => coeff_3: MsbU16,
    register [0xA8; 2] => coeff_4: MsbU16,
    register [0xAA; 2] => coeff_5: MsbU16,
    register [0xAC; 2] => coeff_6: MsbU16,
    register [0xAE; 2] => serial_and_crc: MsbU16,
    //register [0xAF; 1] => crc: Crc,

    register [0x40; 1] => d1_osr_0: u8,
    register [0x42; 1] => d1_osr_1: u8,
    register [0x44; 1] => d1_osr_2: u8,
    register [0x46; 1] => d1_osr_3: u8,
    register [0x48; 1] => d1_osr_4: u8,

    register [0x50; 1] => d2_osr_0: u8,
    register [0x52; 1] => d2_osr_1: u8,
    register [0x54; 1] => d2_osr_2: u8,
    register [0x56; 1] => d2_osr_3: u8,
    register [0x58; 1] => d2_osr_4: u8    
  }
);

#[derive(PackedStruct, Debug, Copy, Clone)]
#[packed_struct(endian="msb")]
pub struct CalibrationData {
    pub coeff_1: u16,
    pub coeff_2: u16,
    pub coeff_3: u16,
    pub coeff_4: u16,
    pub coeff_5: u16,
    pub coeff_6: u16
}

pub type Ms5611OnI2CBus<B> = Ms5611<<B as Bus>::SystemApi, <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::Registers>;

#[derive(Clone, Copy)]
pub struct Ms5611Factory {
    addresses: [I2CAddress; 2]
}

impl Default for Ms5611Factory {
    fn default() -> Self {
        Ms5611Factory {
            addresses: [
                I2CAddress::address_7bit(0x76),
                I2CAddress::address_7bit(0x77)
            ]
        }
    }
}

impl<B> DeviceI2CDetection<Ms5611OnI2CBus<B>, B, I2CDeviceRegisters<B>> for Ms5611Factory 
    where B: Bus + 'static,
{
	fn get_addresses(&self) -> &[I2CAddress] {
        &self.addresses
    }

	fn new(args: I2CDeviceRegisters<B>) -> Result<Ms5611OnI2CBus<B>, PeripheryError> {
        let sensor = Ms5611 {
            system: args.system_api,
            bus: args.device_bus
        };

        sensor.read_calibration_data()?;

        Ok(sensor)
    }
}

#[derive(Clone)]
pub struct Ms5611<S, B> where S: SystemApi, B: DeviceRegisterBus {
    system: S,
    bus: B
}

impl<S, B> Ms5611<S, B> where S: SystemApi, B: DeviceRegisterBus {
    pub fn registers<'b>(&'b self) -> Ms5611Registers<'b, B> {
        Ms5611Registers::new(&self.bus)
    }

    pub fn read_calibration_data(&self) -> Result<CalibrationData, PeripheryError> {
        let factory_data = **try!(self.registers().factory_data().read());
        let coeff_1 = **try!(self.registers().coeff_1().read());
        let coeff_2 = **try!(self.registers().coeff_2().read());
        let coeff_3 = **try!(self.registers().coeff_3().read());
        let coeff_4 = **try!(self.registers().coeff_4().read());
        let coeff_5 = **try!(self.registers().coeff_5().read());
        let coeff_6 = **try!(self.registers().coeff_6().read());
        let serial_and_crc = **try!(self.registers().serial_and_crc().read());

        let sensor_crc = (serial_and_crc & 0x000F) as u8;

        let rom = [
            factory_data,
            coeff_1,
            coeff_2,
            coeff_3,
            coeff_4,
            coeff_5,
            coeff_6,
            serial_and_crc
        ];

        if let Ok(calculated_crc) = calculate_crc(&rom) {

            if sensor_crc != calculated_crc {
                return Err(PeripheryError::CrcMismatch { expected: sensor_crc as u16, calculated: calculated_crc as u16 });
            }

            Ok(CalibrationData {
                coeff_1: coeff_1,
                coeff_2: coeff_2,
                coeff_3: coeff_3,
                coeff_4: coeff_4,
                coeff_5: coeff_5,
                coeff_6: coeff_6
            })
        } else {
            Err(PeripheryError::ReadParseError)
        }
    }

    pub fn start_measurement(&self, data: DataRequest, oversampling: Oversampling) -> Result<(), PeripheryError> {
        let registers = self.registers();

        let start_measurement_register = match data {
            DataRequest::Pressure => {
                match oversampling {
                    Oversampling::UltraLowPower => registers.d1_osr_0(),
                    Oversampling::LowPower => registers.d1_osr_1(),
                    Oversampling::Standard => registers.d1_osr_2(),
                    Oversampling::HighRes => registers.d1_osr_3(),
                    Oversampling::UltraHighRes => registers.d1_osr_4()
                }
            },
            DataRequest::Temperature => {
                match oversampling {
                    Oversampling::UltraLowPower => registers.d2_osr_0(),
                    Oversampling::LowPower => registers.d2_osr_1(),
                    Oversampling::Standard => registers.d2_osr_2(),
                    Oversampling::HighRes => registers.d2_osr_3(),
                    Oversampling::UltraHighRes => registers.d2_osr_4()
                }
            }
        };

        try!(start_measurement_register.write(&0));
        Ok(())
    }

    /// Start the measurement, wait for the correct time period and read the data.
    pub fn sample_raw_data(&self, data: DataRequest, oversampling: Oversampling) -> Result<u32, PeripheryError> {
        try!(self.start_measurement(data, oversampling));

        self.system.get_sleep()?.sleep_ms(oversampling.get_conversion_ms_delay());

        let v = try!(self.registers().adc().read());
        Ok(**v)
    }

    pub fn get_delta_temperature(&self, calibration_data: &CalibrationData, oversampling: Oversampling) -> Result<i32, PeripheryError> {
        let t = try!(self.sample_raw_data(DataRequest::Temperature, oversampling));
        Ok(calculate_delta_temperature(calibration_data, t))
    }

    pub fn get_temperature(&self, calibration_data: &CalibrationData, oversampling: Oversampling) -> Result<AmbientTemperature, PeripheryError> {
        let dt = try!(self.get_delta_temperature(calibration_data, oversampling));
        let t = calculate_temperature_int(calibration_data, dt);
        let t = AmbientTemperature::from_temperature(Temperature::from_degrees_celsius((t as f32) / 100.0));
        Ok(t)
    }

    pub fn get_pressure(&self, calibration_data: &CalibrationData, oversampling: Oversampling) -> Result<AtmosphericPressure, PeripheryError> {
        let pressure = try!(self.sample_raw_data(DataRequest::Pressure, oversampling));
        let t = try!(self.sample_raw_data(DataRequest::Temperature, oversampling));

        Ok(calculate_pressure(calibration_data, t, pressure))
    }

    pub fn new_sampler(&self, temperature_cycles: usize, oversampling: Oversampling) -> Result<ContinousSampler, PeripheryError> {
        if temperature_cycles == 0 {
            return Err(PeripheryError::UnsupportedFieldValue);
        }

        let calib = try!(self.read_calibration_data());

        let samp = ContinousSampler {
            state: State::New,
            sample_temperature_every_cycles: temperature_cycles,
            cycle: 0,
            oversampling: oversampling,
            calibration: calib,

            raw_temperature: 0,
            raw_pressure: 0,
            pressure: None
        };
        Ok(samp)
    } 
}

pub fn calculate_delta_temperature(calibration_data: &CalibrationData, d2: u32) -> i32 {
    (d2 as i32) - ((calibration_data.coeff_5 as i32) << 8)
}

pub fn calculate_temperature_int(calibration_data: &CalibrationData, delta_temperature: i32) -> i32 {
    let dt = delta_temperature;

    let t = 2000 + ((dt * (calibration_data.coeff_6 as i32)) >> 23);
    t
}

pub fn calculate_pressure_temp_offset(calibration_data: &CalibrationData, delta_temperature: i32) -> i64 {
    ((calibration_data.coeff_2 as i64) << 16) + ((calibration_data.coeff_4 as i64 * delta_temperature as i64) >> 7)
}

pub fn calculate_sensitivity(calibration_data: &CalibrationData, delta_temperature: i32) -> i64 {
    ((calibration_data.coeff_1 as i64) << 15) + ((calibration_data.coeff_3 as i64 * delta_temperature as i64) >> 8)
}

pub fn calculate_compensated_pressure(d1: u32, sensitivity: i64, pressure_temp_offset: i64) -> i32 {
    let p = (((d1 as i64 * sensitivity) >> 21) - pressure_temp_offset) >> 15;    
    p as i32
}

pub fn calculate_pressure(calibration_data: &CalibrationData, raw_temperature: u32, raw_pressure: u32) -> AtmosphericPressure {
    let dt = calculate_delta_temperature(calibration_data, raw_temperature);
    let off = calculate_pressure_temp_offset(calibration_data, dt);
    let sens = calculate_sensitivity(calibration_data, dt);
    let p = calculate_compensated_pressure(raw_pressure, sens, off);
    
    let p = AtmosphericPressure::from_pressure(Pressure::from_pascal(p as f32));
    p
}


impl<S, B> Device for Ms5611<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_ambient_temperature_sensor(&self) -> Option<&AmbientTemperatureSensor> {
        Some(self)
    }    

    fn get_atmospheric_pressure_sensor(&self) -> Option<&AtmosphericPressureSensor> {
        Some(self)
    }

    fn description(&self) -> Cow<str> {
        "MS5611 barometer".into()
    }

    fn get_registers_cli(&self) -> Option<DeviceBusCli> {
        let mut c = DeviceBusCli::new();
        c.with_registers(self.registers());
        Some(c)
    }

    fn id(&self) -> Cow<str> {
        "ms5611".into()
    }
}

impl<S, B> AmbientTemperatureSensor for Ms5611<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_ambient_temperature(&self) -> Result<AmbientTemperature, PeripheryError> {
        let c = try!(self.read_calibration_data());
        self.get_temperature(&c, Oversampling::Standard)
    }
}

impl<S, B> AtmosphericPressureSensor for Ms5611<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_atmospheric_pressure(&self) -> Result<AtmosphericPressure, PeripheryError> {
        let c = try!(self.read_calibration_data());
        self.get_pressure(&c, Oversampling::Standard)
    }
}


#[derive(PartialEq)]
enum State {
    New,
    ReadTemperature,
    ReadPressure
}

pub struct ContinousSampler {
    state: State,
    sample_temperature_every_cycles: usize,
    cycle: usize,
    oversampling: Oversampling,
    calibration: CalibrationData,

    raw_temperature: u32,
    raw_pressure: u32,
    pressure: Option<AtmosphericPressure>
}

#[derive(Copy, Clone, Debug)]
pub struct SamplingCycle {
    pub sleep_ms_required: usize
}

impl ContinousSampler {
    pub fn sample<S, B>(&mut self, sensor: &Ms5611<S, B>) -> Result<SamplingCycle, PeripheryError> where S: SystemApi, B: DeviceRegisterBus {
        match self.state {
            State::ReadTemperature => {
                self.raw_temperature = **try!(sensor.registers().adc().read());

                try!(sensor.start_measurement(DataRequest::Pressure, self.oversampling));
                self.state = State::ReadPressure;
                self.cycle = 0;
            },
            State::New | State::ReadPressure => {
                if self.state != State::New {
                    self.raw_pressure = **try!(sensor.registers().adc().read());
                    self.cycle += 1;

                    self.pressure = Some(calculate_pressure(&self.calibration, self.raw_temperature, self.raw_pressure));
                }
                
                if self.cycle >= self.sample_temperature_every_cycles {
                    try!(sensor.start_measurement(DataRequest::Temperature, self.oversampling));
                    self.state = State::ReadTemperature;
                } else {
                    try!(sensor.start_measurement(DataRequest::Pressure, self.oversampling));
                    self.state = State::ReadPressure;
                }
            }
        }

        Ok(SamplingCycle {
            sleep_ms_required: self.oversampling.get_conversion_ms_delay() as usize
        })
    }

    pub fn get_pressure(&self) -> Option<AtmosphericPressure> {
        self.pressure
    }
}




fn calculate_crc(rom_contents: &[u16]) -> Result<u8, ()> {
    if rom_contents.len() != 8 { return Err(()); }

    let mut crc: u16 = 0;

    for i in 0..16 {
        let r = if i == 15 {
            0
        } else {
            rom_contents[i >> 1]
        };

        if (i % 2) == 1 {
            crc ^= (r & 0x00FF) as u16;
        } else {
            crc ^= (r >> 8) as u16;
        }

        let mut k = 8;
        loop {
            if (crc & 0x8000) == 0x8000 {
                crc = (crc << 1) ^ 0x3000;
            } else {
                crc = crc << 1;
            }

            k -= 1;

            if k == 0 { break; }
        }

    }

    Ok(((crc >> 12) & 0xF) as u8)
}

#[test]
pub fn crc_test() {
    // the last byte should be zeroed in CRC calculation. Simulated non-null input byte, just like
    // on the real sensor - LSB 4 bits contain the CRC itself!
    //let rom = [0x3132,0x3334,0x3536,0x3738,0x3940,0x4142,0x4344,0x4500];
    let rom = [0x3132,0x3334,0x3536,0x3738,0x3940,0x4142,0x4344, 0x45AC];

    let crc = calculate_crc(&rom).unwrap();
    assert_eq!(0x0B, crc);
}


#[test]
pub fn test_example() {
    let calib = CalibrationData {
        coeff_1: 40127,
        coeff_2: 36924,
        coeff_3: 23317,
        coeff_4: 23282,
        coeff_5: 33464,
        coeff_6: 28312
    };

    let d1 = 9085466;
    let d2 = 8569150;

    let dt = calculate_delta_temperature(&calib, d2);
    assert_eq!(2366, dt);

    let temp = calculate_temperature_int(&calib, dt);
    assert_eq!(2007, temp);

    let off = calculate_pressure_temp_offset(&calib, dt);
    assert_eq!(2420281617, off);

    let sens = calculate_sensitivity(&calib, dt);
    assert_eq!(1315097036, sens);

    let p = calculate_compensated_pressure(d1, sens, off);
    assert_eq!(100009, p);
}