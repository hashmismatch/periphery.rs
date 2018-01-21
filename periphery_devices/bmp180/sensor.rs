use periphery_core::*;
use periphery_core::prelude::v1::*;
use periphery_core::terminal_cli::*;

use registers::*;

use packed_struct::prelude::*;

type MsbI16 = MsbInteger<i16, packed_bits::Bits16, Integer<i16, packed_bits::Bits16>>;
type MsbU16 = MsbInteger<u16, packed_bits::Bits16, Integer<u16, packed_bits::Bits16>>;

registers!(
  chip Bmp180Registers {
  	register [0xAA; 2] => ac1: MsbI16,
  	register [0xAC; 2] => ac2: MsbI16,
  	register [0xAE; 2] => ac3: MsbI16,
  	register [0xB0; 2] => ac4: MsbU16,
  	register [0xB2; 2] => ac5: MsbU16,
  	register [0xB4; 2] => ac6: MsbU16,
  	register [0xB6; 2] => b1 : MsbI16,
  	register [0xB8; 2] => b2 : MsbI16,
  	register [0xBA; 2] => mb : MsbI16,
  	register [0xBC; 2] => mc : MsbI16,
  	register [0xBE; 2] => md : MsbI16,

    register [0xD0; 1] => id: u8,

    register [0xE0; 1] => soft_reset: ResetRegister,

    register [0xF4; 1] => measurement_control: MeasurementControlRegister,
    register [0xF6; 2] => measurement_u16: MeasurementValueMsb,
    register [0xF8; 1] => measurement_xlsb: u8    
  }
);

pub type Bmp180OnI2CBus<B> = Bmp180<<B as Bus>::SystemApi, <<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::Registers>;

#[derive(Clone, Copy)]
pub struct Bmp180Factory {
    addresses: [I2CAddress; 1]
}

impl Default for Bmp180Factory {
    fn default() -> Self {
        Bmp180Factory {
            addresses: [I2CAddress::address_7bit(0x77)]
        }
    }
}

impl<B> DeviceI2CDetection<Bmp180OnI2CBus<B>, B, I2CDeviceRegisters<B>> for Bmp180Factory 
    where B: Bus + 'static,
{
	fn get_addresses(&self) -> &[I2CAddress] {
        &self.addresses
    }

	fn new(args: I2CDeviceRegisters<B>) -> Result<Bmp180OnI2CBus<B>, PeripheryError> {
        let sensor = Bmp180 {
            system: args.system_api,
            bus: args.device_bus
        };

        let id = sensor.registers().id().read()?;
        
        if id == 0x55 {
            return Ok(sensor);
        }

        Err(PeripheryError::UnsupportedFieldValue)
    }
}


#[derive(Clone)]
pub struct Bmp180<S, B> where S: SystemApi, B: DeviceRegisterBus {
    system: S,
    bus: B
}

impl<S, B> Bmp180<S, B> where S: SystemApi, B: DeviceRegisterBus {
    #[inline]
    pub fn registers<'a>(&'a self) -> Bmp180Registers<'a, B> {
        Bmp180Registers::new(&self.bus)
    }

    pub fn read_calibration_coefficients(&self) -> Result<CalibrationCoefficients, PeripheryError> {
        Ok(CalibrationCoefficients {
            ac1: **self.registers().ac1().read()?,
            ac2: **self.registers().ac2().read()?,
            ac3: **self.registers().ac3().read()?,
            ac4: **self.registers().ac4().read()?,
            ac5: **self.registers().ac5().read()?,
            ac6: **self.registers().ac6().read()?,
            b1: **self.registers().b1().read()?,
            b2: **self.registers().b2().read()?,
            mb: **self.registers().mb().read()?,
            mc: **self.registers().mc().read()?,
            md: **self.registers().md().read()?
        })
    }

    pub fn read_temperature(&self, calib: &CalibrationCoefficients) -> Result<BMP180Temperature, PeripheryError> {
        let control = MeasurementControlRegister {
            oss: PressureOversamplingRatio::Times1,
            sco: ConversionStatus::Running,
            measurement: MeasurementType::Temperature
        };
        try!(self.registers().measurement_control().write(&control));

        self.system.get_sleep()?.sleep_ms(control.oss.get_required_ms_wait_after_measurement());
        
        let temp = try!(self.registers().measurement_u16().read());

        bmp180_calc_temperature(calib, temp.value as i32)
    }

    pub fn read_pressure(&self, calib: &CalibrationCoefficients, oversampling: PressureOversamplingRatio, temperature: &BMP180Temperature) -> Result<BMP180Pressure, PeripheryError> {
        let control = MeasurementControlRegister {
            oss: oversampling,
            sco: ConversionStatus::Running,
            measurement: MeasurementType::Pressure
        };

        self.registers().measurement_control().write(&control)?;

        self.system.get_sleep()?.sleep_ms(oversampling.get_required_ms_wait_after_measurement());
        
        let up = {
            let m = self.registers().measurement_u16().read()?.value;
            let xlsb = self.registers().measurement_xlsb().read()?;

            ((m as u32) << 8) + (xlsb as u32)
        };

        let up = up >> (8 - oversampling as u32);

        bmp180_calc_pressure(calib, oversampling, up as i32, temperature)
    }    

    pub fn reset_sensor(&self) -> Result<(), PeripheryError> {
        let reset = ResetRegister {
            state: ResetState::TriggerReset
        };
        self.registers().soft_reset().write(&reset)?;
		Ok(())
    }
}





impl<S, B> AmbientTemperatureSensor for Bmp180<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_ambient_temperature(&self) -> Result<AmbientTemperature, PeripheryError> {
        let c = try!(self.read_calibration_coefficients());
        let t = try!(self.read_temperature(&c));
        Ok(t.to_temperature())
    }
}

impl<S, B> AtmosphericPressureSensor for Bmp180<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_atmospheric_pressure(&self) -> Result<AtmosphericPressure, PeripheryError> {
        let c = try!(self.read_calibration_coefficients());
        let t = try!(self.read_temperature(&c));
        let p = try!(self.read_pressure(&c, PressureOversamplingRatio::Times1, &t));
        Ok(p.to_atmospheric_pressure())
    }
}





impl<S, B> Device for Bmp180<S, B> where S: SystemApi, B: DeviceRegisterBus {
    fn get_ambient_temperature_sensor(&self) -> Option<&AmbientTemperatureSensor> {
        Some(self)
    }

    fn get_atmospheric_pressure_sensor(&self) -> Option<&AtmosphericPressureSensor> {
        Some(self)
    }
    
    fn description(&self) -> Cow<str> {
        "BMP180 digital pressure sensor".into()
    }

    fn get_registers_cli(&self) -> Option<DeviceBusCli> {
        let mut c = DeviceBusCli::new();
        c.with_registers(self.registers());
        Some(c)
    }

    fn id(&self) -> Cow<str> {
        "bmp180".into()
    }

	fn get_cli(&self) -> Option<&DeviceCli> {
		Some(self)
	}

    fn init_after_detection(&self) -> Result<bool, PeripheryError> {
        self.reset_sensor()?;
        Ok(true)
	}
}


impl<S, B> DeviceCli for Bmp180<S, B> where S: SystemApi, B: DeviceRegisterBus {
	fn execute_cli(&self, exec: &mut PrefixedExecutor) {
        if let Some(mut ctx) = exec.command(&"calibration_coefficients/read") {
            if let Ok(calib) = self.read_calibration_coefficients() {
                write!(ctx.get_terminal(), "{:?}\r\n", calib);
            }
        }
    }
}






#[derive(Debug, Copy, Clone)]
pub struct CalibrationCoefficients {
    pub ac1: i16,
    pub ac2: i16,
    pub ac3: i16,
    pub ac4: u16,
    pub ac5: u16,
    pub ac6: u16,
    pub b1: i16,
    pub b2: i16,
    pub mb: i16,
    pub mc: i16,
    pub md: i16
}


fn bmp180_calc_temperature(calib: &CalibrationCoefficients, ut: i32) -> Result<BMP180Temperature, PeripheryError> {
    let x1 = ((ut - calib.ac6 as i32) * (calib.ac5 as i32)) >> 15;
    let x1_v = x1 + (calib.md as i32);
    if x1_v == 0 { return Err(PeripheryError::CalculationError); }
    let x2 = ((calib.mc as i32) << 11) / x1_v;

    let b5 = x1 + x2;

    let t = (b5 + 8) >> 4;

    Ok(BMP180Temperature {
        temp: t as i16,
        b5: b5
    })
}


fn bmp180_calc_pressure(calib: &CalibrationCoefficients, oversampling: PressureOversamplingRatio, up: i32, temperature: &BMP180Temperature) -> Result<BMP180Pressure, PeripheryError> {
    let oss = oversampling as u8;

    let b6 = temperature.b5 - 4000;
    let x1 = (calib.b2 as i32 * ((b6 * b6 ) >> 12)) >> 11;      
    let x2 = (calib.ac2 as i32 * b6) >> 11;
    let x3 = x1 + x2;
    
    let b3 = (((((calib.ac1 as i64) * 4) + x3 as i64) << oss) + 2) / 4;     
    let x1 = (calib.ac3 as i32 * b6) >> 13;
    let x2 = (calib.b1 as i32 * ((b6 * b6) >> 12)) >> 16;
    let x3 = ((x1 + x2) + 2) >> 2;

    let b4 = ((calib.ac4 as u64) * (x3 + 32768) as u64) >> 15;
    let b7 = (up as i64 - b3) * (50000 >> oss);

    if b4 == 0 { return Err(PeripheryError::CalculationError); }

    let p = if b7 < 0x80000000 {
        (b7 * 2) / b4 as i64
    } else {
        (b7 / b4 as i64) * 2
    };

    let x1 = (p >> 8) * (p >> 8);
    let x1 = (x1 * 3038) >> 16;
    let x2 = (-7357 * p) >> 16;
    let p = p + ((x1 + x2 + 3791) >> 4);

    Ok(BMP180Pressure {
        pressure: p as i32
    })
}   




#[derive(Copy, Clone, Debug)]
pub struct BMP180Temperature {
    /// [0.1C]
    pub temp: i16,
    b5: i32
}

impl BMP180Temperature {
    pub fn to_temperature(&self) -> AmbientTemperature {
        AmbientTemperature::from_temperature(Temperature::from_degrees_celsius((self.temp as f32) / 10.0))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BMP180Pressure {
    /// [Pa]
    pub pressure: i32
}

impl BMP180Pressure {
    pub fn to_atmospheric_pressure(&self) -> AtmosphericPressure {
        AtmosphericPressure::from_pressure(Pressure::from_pascal(self.pressure as f32))
    }
}


#[cfg(test)]
#[test]
pub fn test_bmp180_calc() {
    let calib = CalibrationCoefficients {
        ac1: 408,
        ac2: -72,
        ac3: -14383,
        ac4: 32741,
        ac5: 32757,
        ac6: 23153,
        b1: 6190,
        b2: 4,
        mb: -32768,
        mc: -8711,
        md: 2868
    };

    let ut = 27898;

    let tt = bmp180_calc_temperature(&calib, ut).unwrap();
    let t = tt.temp;
    assert_eq!(150, t);
    
    // Off by one from the one in the PDF sample! PDF states 2399 as the correct result.
    assert_eq!(2400, tt.b5);

    let up = 23843;

    let p = bmp180_calc_pressure(&calib, PressureOversamplingRatio::Times1, up, &tt).unwrap();
    assert_eq!(69964, p.pressure);

}