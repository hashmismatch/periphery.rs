use periphery_core::*;
use periphery_core::prelude::v1::*;
use periphery_core::terminal_cli::*;

use registers::*;

use packed_struct::*;

registers!(
  chip Bmp280Registers {
    register [0x88; 24] => calibration_coefficients: CalibrationCoefficients,
    
    register [0xD0; 1] => id: u8,

    register [0xF3; 1] => status: StatusRegister,
    register [0xF4; 1] => control_measurement: ControlMeasurementRegister,
    register [0xF5; 1] => config: ConfigurationRegister,

    register [0xE0; 1] => soft_reset: ResetRegister,

    register [0xF7; 3] => pressure: MeasurementValue,
    register [0xFA; 3] => temperature: MeasurementValue
  }
);

pub type Bmp280OnI2CBus<B> = Bmp280<<<<B as Bus>::I2C as I2CBus>::DeviceFactory as I2CBusDeviceFactory>::Registers>;

#[derive(Clone, Copy)]
pub struct Bmp280Factory {
    addresses: [I2CAddress; 1]
}

impl Default for Bmp280Factory {
    fn default() -> Self {
        Bmp280Factory {
            addresses: [I2CAddress::address_7bit(0x76)]
        }
    }
}

impl<B> DeviceI2CDetection<Bmp280OnI2CBus<B>, B, I2CDeviceRegisters<B>> for Bmp280Factory 
    where B: Bus + 'static,
{
	fn get_addresses(&self) -> &[I2CAddress] {
        &self.addresses
    }

	fn new(args: I2CDeviceRegisters<B>) -> Result<Bmp280OnI2CBus<B>, PeripheryError> {
        let sensor = Bmp280 {
            bus: args.device_bus
        };

        let id = sensor.registers().id().read()?;
        
        if id == 0x58 {
            return Ok(sensor);
        }

        Err(PeripheryError::UnsupportedFieldValue)
    }
}

#[derive(Clone)]
pub struct Bmp280<B> where B: DeviceRegisterBus {
    bus: B,
}

impl<B> Bmp280<B> where B: DeviceRegisterBus {
    #[inline]
    pub fn registers<'a>(&'a self) -> Bmp280Registers<'a, B> {
        Bmp280Registers::new(&self.bus)
    }

    pub fn read_calibration_coefficients(&self) -> Result<CalibrationCoefficients, PeripheryError> {
        self.registers().calibration_coefficients().read()
    }
}


impl<B> Device for Bmp280<B> where B: DeviceRegisterBus {
    fn description(&self) -> Cow<str> {
        "BMP280 digital pressure sensor".into()
    }

    fn get_registers_cli(&self) -> Option<DeviceBusCli> {
        let mut c = DeviceBusCli::new();
        c.with_registers(self.registers());
        Some(c)
    }

    fn id(&self) -> Cow<str> {
        "bmp280".into()
    }

    fn get_ambient_temperature_sensor(&self) -> Option<&AmbientTemperatureSensor> {
        Some(self)
    }

    fn get_atmospheric_pressure_sensor(&self) -> Option<&AtmosphericPressureSensor> {
        Some(self)
    }
    
    fn init_after_detection(&self) -> Result<bool, PeripheryError> {
        self.registers().control_measurement().write(&ControlMeasurementRegister {
            oversampling_temperature: Oversampling::Times1,
            oversampling_pressure: Oversampling::Times8,
            power_mode: PowerMode::NormalMode
        })?;

        self.registers().config().write(&ConfigurationRegister {
            standby_time: StandbyTime::Standby0_5ms,
            iir_filter: IirFilter::Filter2,
            enable_3wire_spi: false
        })?;

		Ok(true)
	}
}

impl<B> AmbientTemperatureSensor for Bmp280<B> where B: DeviceRegisterBus {
    fn get_ambient_temperature(&self) -> Result<AmbientTemperature, PeripheryError> {
        let c = self.read_calibration_coefficients()?;
        let t = self.registers().temperature().read()?;
        let t = calculate_temperature(&c, *t.value as i32);

        Ok(t.to_ambient_temperature())
    }
}

impl<B> AtmosphericPressureSensor for Bmp280<B> where B: DeviceRegisterBus {
    fn get_atmospheric_pressure(&self) -> Result<AtmosphericPressure, PeripheryError> {
        let c = self.read_calibration_coefficients()?;
        let t = self.registers().temperature().read()?;
        let t = calculate_temperature(&c, *t.value as i32);
        let p = self.registers().pressure().read()?;

        let p = calculate_pressure(&c, *p.value as i32, &t)?;

        Ok(p.to_atmospheric_pressure())
    }
}

#[derive(Copy, Clone, Debug)]
struct Bmp280Temperature {
    t_fine: i32,
    temperature: i32
}

impl Bmp280Temperature {
    fn to_ambient_temperature(&self) -> AmbientTemperature {
        let t = Temperature::from_degrees_celsius(self.temperature as f32 * 0.01);
        AmbientTemperature::from_temperature(t)
    }
}


fn calculate_temperature(calib: &CalibrationCoefficients, adc_t: i32) -> Bmp280Temperature {
    let v1 = ((((adc_t>>3) - (calib.t1 << 1) as i32)) * (calib.t2 as i32)) >> 11;
    let v2 = ((((adc_t>>4) - (calib.t1 as i32)) * ((adc_t>>4) - (calib.t1 as i32)) >> 12) * (calib.t3 as i32)) >> 14;
    let t_fine = v1+v2;
    
    Bmp280Temperature {
        t_fine: t_fine,
        temperature: (t_fine * 5 + 128) >> 8
    }
}

#[derive(Copy, Clone, Debug)]
struct Bmp280Pressure {
    pressure: u32
}

impl Bmp280Pressure {
    fn to_atmospheric_pressure(&self) -> AtmosphericPressure {
        AtmosphericPressure::from_pressure(Pressure::from_pascal(self.pressure as f32 / 256.0))
    }
}

fn calculate_pressure(calib: &CalibrationCoefficients, adc_p: i32, temperature: &Bmp280Temperature) -> Result<Bmp280Pressure, PeripheryError> {
    let v1: i64 = (temperature.t_fine as i64) - 128000;
    let v2: i64 = (v1 * v1) * (calib.p6 as i64);
    let v2 = v2 + ((v1*(calib.p5 as i64))<<17);
    let v2 = v2 + ((calib.p4 as i64)<<35);
    let v1 = ((v1 * v1 * (calib.p3 as i64))>>8) + ((v1 * (calib.p2 as i64))<<12);
    let v1 = ((((1)<<47)+v1))*(calib.p1 as i64)>>33;
    if v1 == 0 {
        return Err(PeripheryError::CalculationError);
    }
    let p: i64 = 1048576-(adc_p as i64);
    let p = (((p<<31)-v2)*3125)/v1;
    let v1 = ((calib.p9 as i64) * (p>>13) * (p>>13)) >> 25;
    let v2 = ((calib.p8 as i64) * p) >> 19;
    let p = ((p + v1 + v2) >> 8) + ((calib.p7 as i64)<<4);
    
    Ok(Bmp280Pressure { pressure: p as u32})
} 

#[test]
#[cfg(test)]
fn test_bmp280_calculation() {
    let calib = CalibrationCoefficients {
        t1: 27504,
        t2: 26435,
        t3: -1000,
        p1: 36477,
        p2: -10685,
        p3: 3024,
        p4: 2855,
        p5: 140,
        p6: -7,
        p7: 15500,
        p8: -14600,
        p9: 6000
    };

    let t = calculate_temperature(&calib, 519888);
    assert_eq!(2508, t.temperature);

    assert_eq!(25.08, t.to_ambient_temperature().get_temperature().get_degrees_celsius());

    let p = calculate_pressure(&calib, 415148, &t).unwrap();
    assert_eq!(25767233, p.pressure); // floating point calculation from documentation: 25767236

    assert!((100653.0 - p.to_atmospheric_pressure().get_pressure().get_pascal()).abs() < 1.0)
}