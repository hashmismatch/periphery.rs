//! Physical SI units for sensors to report their readings.

use prelude::v1::*;

#[derive(Copy, Clone)]
pub struct Temperature {
	val: f32,
	unit: TemperatureUnit
}

#[derive(Copy, Clone, Debug)]
pub enum TemperatureUnit {
	DegreesCelsius,
	Kelvin
}

impl Temperature {
	#[inline]
	pub fn from_degrees_celsius(c: f32) -> Temperature {
		Temperature { val: c, unit: TemperatureUnit::DegreesCelsius }
	}
	#[inline]
	pub fn from_kelvin(k: f32) -> Temperature {
		Temperature { val: k, unit: TemperatureUnit::Kelvin }
	}


	pub fn get_degrees_celsius(&self) -> f32 {
		match self.unit {
			TemperatureUnit::DegreesCelsius => self.val,
			TemperatureUnit::Kelvin => self.val - 273.15
		}
	}

	pub fn get_kelvin(&self) -> f32 {
		match self.unit {
			TemperatureUnit::DegreesCelsius => self.val + 273.15,
			TemperatureUnit::Kelvin => self.val
		}
	}
	#[inline]
	pub fn to_degrees_celsius(&self) -> Temperature {
		Temperature::from_degrees_celsius(self.get_degrees_celsius())
	}	#[inline]

	pub fn to_kelvin(&self) -> Temperature {
		Temperature::from_kelvin(self.get_kelvin())
	}

	fn self_fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self.unit {
			TemperatureUnit::DegreesCelsius => write!(f, "{} deg C", self.get_degrees_celsius()),
			TemperatureUnit::Kelvin => write!(f, "{} K", self.get_kelvin())
		}
	}
}

impl Debug for Temperature {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}

impl Display for Temperature {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}


#[derive(Copy, Clone)]
pub struct AmbientTemperature {
	val: Temperature
}

impl AmbientTemperature {
	#[inline]
	pub fn from_temperature(t: Temperature) -> AmbientTemperature {
		AmbientTemperature { val: t }
	}
	#[inline]
	pub fn get_temperature(&self) -> Temperature {
		self.val
	}

	fn self_fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Ambient temperature: {}", self.val)
	}
}

impl Debug for AmbientTemperature {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}

impl Display for AmbientTemperature {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}


#[derive(Copy, Clone)]
pub struct ColorTemperature {
	val: Temperature
}

impl ColorTemperature {
	#[inline]
	pub fn from_temperature(t: Temperature) -> ColorTemperature {
		ColorTemperature { val: t }
	}
	#[inline]
	pub fn get_temperature(&self) -> Temperature {
		self.val
	}

	fn self_fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Color temperature: {}", self.val)
	}
}

impl Debug for ColorTemperature {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}

impl Display for ColorTemperature {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}




#[derive(Copy, Clone)]
pub struct Percentage(f32);

impl Percentage {
	#[inline]
	pub fn from_percentage(p: f32) -> Percentage {
		Percentage(p)
	}
	#[inline]
	pub fn get_percentage(&self) -> f32 {
		let Percentage(p) = *self;
		p
	}
}

impl Debug for Percentage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}%", self.get_percentage())
    }
}

impl Display for Percentage {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	write!(f, "{}%", self.get_percentage())
    }
}


#[derive(Copy, Clone)]
pub struct RelativeHumidity { percentage: Percentage }

impl RelativeHumidity {
	#[inline]
	pub fn from_percentage(p: Percentage) -> RelativeHumidity {
		RelativeHumidity { percentage: p }
	}
	#[inline]
	pub fn get_percentage(&self) -> Percentage {
		self.percentage
	}
}

impl Debug for RelativeHumidity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{:?} relative humidity", self.percentage)
    }
}

impl Display for RelativeHumidity {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	write!(f, "{} relative humidity", self.percentage)
    }
}

#[derive(Copy, Clone)]
pub struct GForce(f32);

impl GForce {
	#[inline]
	pub fn from_g_force(g: f32) -> Self {
		GForce(g)
	}
	#[inline]
	pub fn std_gravity() -> Self {
		Self::from_g_force(1.0)
	}
	#[inline]
	pub fn get_g_force(&self) -> f32 {
		self.0
	}
	#[inline]
	pub fn zero() -> GForce {
		GForce(0.0)
	}
}

impl Debug for GForce {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} g", self.get_g_force())
    }
}

impl Display for GForce {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	write!(f, "{} g", self.get_g_force())
    }
}

impl Default for GForce {
	#[inline]
	fn default() -> Self {
		Self::from_g_force(0.0)
	}
}

#[derive(Copy, Clone, Default)]
pub struct Acceleration3 {
	pub x: GForce,
	pub y: GForce,
	pub z: GForce
}

impl Acceleration3 {
	#[inline]
	pub fn new(x: GForce, y: GForce, z: GForce) -> Self {
		Acceleration3 {
			x: x,
			y: y,
			z: z
		}
	}
	#[inline]
	pub fn zero() -> Self {
		Self::new(GForce::zero(), GForce::zero(), GForce::zero())
	}
	#[inline]
	pub fn get_x(&self) -> GForce {
		self.x
	}
	#[inline]
	pub fn get_y(&self) -> GForce {
		self.y
	}
	#[inline]
	pub fn get_z(&self) -> GForce {
		self.z
	}
}

impl Debug for Acceleration3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "X: {}, Y: {}, Z: {}", self.get_x(), self.get_y(), self.get_z())
    }
}

impl Display for Acceleration3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	write!(f, "X: {}, Y: {}, Z: {}", self.get_x(), self.get_y(), self.get_z())
    }
}


#[derive(Copy, Clone)]
pub struct Illuminance(f32);

impl Illuminance {
	#[inline]
	pub fn from_lux(l: f32) -> Illuminance {
		Illuminance(l)
	}
	#[inline]
	pub fn get_lux(&self) -> f32 {
		self.0
	}
}

impl Debug for Illuminance {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} lx", self.get_lux())
    }
}

impl Display for Illuminance {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	write!(f, "{} lx", self.get_lux())
    }
}


#[derive(Copy, Clone)]
pub struct Pressure { pascal: f32 }

impl Pressure {
	#[inline]
	pub fn from_pascal(pa: f32) -> Pressure {
		Pressure { pascal: pa }
	}
	#[inline]
	pub fn get_pascal(&self) -> f32 {
		self.pascal
	}
	#[inline]
	pub fn get_hecto_pascal(&self) -> f32 {
		self.get_pascal() * 0.01
	}
}

impl Debug for Pressure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} Pa", self.get_pascal())
    }
}

impl Display for Pressure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	write!(f, "{} Pa", self.get_pascal())
    }
}


#[derive(Copy, Clone)]
pub struct AtmosphericPressure {
	val: Pressure
}

impl AtmosphericPressure {
	#[inline]
	pub fn from_pressure(p: Pressure) -> AtmosphericPressure {
		AtmosphericPressure { val: p }
	}
	#[inline]
	pub fn get_pressure(&self) -> Pressure {
		self.val
	}

	fn self_fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Atmospheric pressure: {}", self.val)
	}

	/// [meters]
	pub fn to_altitude_m(&self) -> f32 {
		(1.0 - (self.get_pressure().get_pascal() / 101325.0).powf(0.190295)) * 44330.0
	}
}

impl Debug for AtmosphericPressure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}

impl Display for AtmosphericPressure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}

impl Default for AtmosphericPressure {
	#[inline]
	fn default() -> Self {
		AtmosphericPressure::from_pressure(Pressure::from_pascal(101325.0))
	}
}

#[derive(Copy, Clone)]
pub struct MagneticFieldStrength { gauss: f32 }

impl MagneticFieldStrength {
	#[inline]
	pub fn from_gauss(gauss: f32) -> MagneticFieldStrength {
		MagneticFieldStrength { gauss: gauss }
	}
	#[inline]
	pub fn get_gauss(&self) -> f32 {
		self.gauss
	}
}

impl Debug for MagneticFieldStrength {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} Gs", self.get_gauss())
    }
}

impl Display for MagneticFieldStrength {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	write!(f, "{} Gs", self.get_gauss())
    }
}

impl Default for MagneticFieldStrength {
	#[inline]
	fn default() -> Self {
		Self::from_gauss(0.0)
	}
}

#[derive(Copy, Clone, Default)]
pub struct MagneticField3 {
	pub x: MagneticFieldStrength,
	pub y: MagneticFieldStrength,
	pub z: MagneticFieldStrength
}

impl MagneticField3 {
	fn self_fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Magnetic field: X {}, Y {}, Z {}", self.x, self.y, self.z)
	}

	#[inline]
	pub fn get_x(&self) -> MagneticFieldStrength {
		self.x
	}

	#[inline]
	pub fn get_y(&self) -> MagneticFieldStrength {
		self.y
	}

	#[inline]
	pub fn get_z(&self) -> MagneticFieldStrength {
		self.z
	}
}

impl Index<usize> for MagneticField3 {
    type Output = MagneticFieldStrength;

    fn index(&self, axis: usize) -> &MagneticFieldStrength {
		match axis {
			0 => &self.x,
			1 => &self.y,
			2 => &self.z,
			_ => panic!("Invalid index for the axis: {}", axis)
		}
	}
}

impl Debug for MagneticField3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}

impl Display for MagneticField3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}

#[derive(Copy, Clone)]
pub struct AngularSpeed { degrees_per_second: f32 }

impl AngularSpeed {
	#[inline]
	pub fn from_degrees_per_second(degrees_per_second: f32) -> AngularSpeed {
		AngularSpeed { degrees_per_second: degrees_per_second }
	}

	#[inline]
	pub fn get_degrees_per_second(&self) -> f32 {
		self.degrees_per_second
	}

	pub fn get_radians_per_second(&self) -> f32 {
		self.degrees_per_second / 57.2958
	}
}

impl Debug for AngularSpeed {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} deg/sec", self.get_degrees_per_second())
    }
}

impl Display for AngularSpeed {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	write!(f, "{} deg/sec", self.get_degrees_per_second())
    }
}

impl Default for AngularSpeed {
	#[inline]
	fn default() -> Self {
		Self::from_degrees_per_second(0.0)
	}
}


#[derive(Copy, Clone, Default)]
pub struct AngularSpeed3 {
	pub x: AngularSpeed,
	pub y: AngularSpeed,
	pub z: AngularSpeed
}

impl AngularSpeed3 {
	#[inline]
	pub fn get_x(&self) -> AngularSpeed {
		self.x
	}

	#[inline]
	pub fn get_y(&self) -> AngularSpeed {
		self.y
	}

	#[inline]
	pub fn get_z(&self) -> AngularSpeed {
		self.z
	}
}

impl AngularSpeed3 {
	fn self_fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Angular speed: X {}, Y {}, Z {}", self.x, self.y, self.z)
	}
}

impl Debug for AngularSpeed3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}

impl Display for AngularSpeed3 {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    	self.self_fmt(f)
    }
}



#[test]
fn test_conversions() {
	let std_pressure: AtmosphericPressure = Default::default();
	assert_eq!(0.0, std_pressure.to_altitude_m());
}

#[test]
fn test_struct_size() {
	use std::mem;
	let s = mem::size_of::<f32>();
	let w = mem::size_of::<AngularSpeed>();

	assert_eq!(s, w);

	let v = mem::size_of::<AngularSpeed3>();

	assert_eq!(s * 3, v);


}




/// Compare to PartialOrd values and return the min.
pub fn partial_min<T: PartialOrd>(a: T, b: T) -> T {
    if a <= b { a } else { b }
}

/// Compare to PartialOrd values and return the min.
pub fn partial_max<T: PartialOrd>(a: T, b: T) -> T {
    if a >= b { a } else { b }
}

/// Clamp a value between some range.
pub fn clamp<T: PartialOrd>(n: T, start: T, end: T) -> T {
    if start <= end {
        if n < start { start } else if n > end { end } else { n }
    } else {
        if n < end { end } else if n > start { start } else { n }
    }
}

pub fn iter_into_partial_min<T: PartialOrd, I: IntoIterator<Item = T>>(iter: I) -> Option<T> {
	let mut min = None;
	for v in iter {
		match min {
			Some(a) => {
				min = Some(partial_min(a, v))
			},
			None => {
				min = Some(v);
			}
		}
	}
	min
}

pub fn iter_into_partial_max<T: PartialOrd, I: IntoIterator<Item = T>>(iter: I) -> Option<T> {
	let mut max = None;
	for v in iter {
		match max {
			Some(a) => {
				max = Some(partial_max(a, v))
			},
			None => {
				max = Some(v);
			}
		}
	}
	max
}

pub fn iter_partial_min<T: PartialOrd, I: Iterator<Item = T>>(iter: I) -> Option<T> {
	let mut min = None;
	for v in iter {
		match min {
			Some(a) => {
				min = Some(partial_min(a, v))
			},
			None => {
				min = Some(v);
			}
		}
	}
	min
}

pub fn iter_partial_max<T: PartialOrd, I: Iterator<Item = T>>(iter: I) -> Option<T> {
	let mut max = None;
	for v in iter {
		match max {
			Some(a) => {
				max = Some(partial_max(a, v))
			},
			None => {
				max = Some(v);
			}
		}
	}
	max
}