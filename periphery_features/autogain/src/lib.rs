extern crate periphery_core;

use periphery_core::prelude::v1::*;
use periphery_core::*;

pub struct GainControllerImpl<M, D, G, Fm, Fs> {
    get_measurement: Fm,
    is_measurement_saturated: Fs,

    last_measurement: Option<M>,
    gain: Option<G>,

    _device: PhantomData<D>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MeasurementSaturation {
    Valid,
    Empty,
    Full
}

#[derive(Clone, Debug)]
pub enum GainControllerError {
    NoGain,
    InProgress,
    PeripheryError(PeripheryError)
}

impl From<PeripheryError> for GainControllerError {
	fn from(err: PeripheryError) -> Self {		
		GainControllerError::PeripheryError(err)
	}
}


pub trait Gain<D>: Sized + Debug + Clone {
    fn retrieve_from_device(device: &D) -> Result<Self, PeripheryError>;
    fn apply_to_device(&self, device: &D) -> Result<(), PeripheryError>;
    fn default_gain() -> Self;
    fn less_gain(&self) -> Option<Self>;
    fn more_gain(&self) -> Option<Self>;

    fn init_gain(device: &D) -> Option<Self> {
        let default_gain = Self::default_gain();
        // error handling?
        match default_gain.apply_to_device(device) {
            Ok(_) => Some(default_gain),
            Err(e) => {
                println!("Error applying the default gain to the sensor: {:?}", e);
                None
            }
        }
        
        /*
        match Self::retrieve_from_device(device) {
            Ok(gain) => { Some(gain) },
            Err(_) => {
                let default_gain = Self::default_gain();
                // error handling?
                match default_gain.apply_to_device(device) {
                    Ok(_) => Some(default_gain),
                    Err(e) => {
                        println!("Error applying the default gain to the sensor: {:?}", e);
                        None
                    }
                }                
            }
        }
        */
    }
}

pub trait GainController<M, D, G> {
    fn get_current_gain(&self) -> &Option<G>;
    fn tick(&mut self, device: &D) -> Result<M, GainControllerError>;
}

impl<M, D, G, Fm, Fs> GainControllerImpl<M, D, G, Fm, Fs> 
    where
        G: Gain<D>,
        D: Device,
        Fm: Fn(&D) -> Result<M, PeripheryError>,
        Fs: Fn(&G, &M) -> MeasurementSaturation
{
    pub fn new(device: &D, get_measurement: Fm, saturation: Fs) -> Self {
        let gain = G::init_gain(device);

        println!("Initialized with gain: {:?}", gain);

        GainControllerImpl {
            get_measurement: get_measurement,
            is_measurement_saturated: saturation,

            last_measurement: None,
            gain: gain,

            _device: Default::default()            
        }
    }
}

impl<M, D, G, Fm, Fs> GainController<M, D, G> for GainControllerImpl<M, D, G, Fm, Fs> 
    where
        G: Gain<D>,
        D: Device,
        Fm: Fn(&D) -> Result<M, PeripheryError>,
        Fs: Fn(&G, &M) -> MeasurementSaturation
{
    fn get_current_gain(&self) -> &Option<G> {
        &self.gain
    }

    fn tick(&mut self, device: &D) -> Result<M, GainControllerError> {
        
        if self.gain.is_none() {
            self.gain = G::init_gain(device);
        }

        let gain = self.gain.clone().ok_or(GainControllerError::NoGain)?;

        let measurement = (self.get_measurement)(device)?;

        match (self.is_measurement_saturated)(&gain, &measurement) {
            MeasurementSaturation::Valid => {
                return Ok(measurement);
            },
            MeasurementSaturation::Empty => {
                // up the gain
                if let Some(higher_gain) = gain.more_gain() {
                    println!("increasing the gain to: {:?}", higher_gain);

                    higher_gain.apply_to_device(device)?;
                    self.gain = Some(higher_gain);
                    
                    // try to throw away the current measurement
                    (self.get_measurement)(device);

                    return Err(GainControllerError::InProgress);
                } else {
                    // we're as high as it goes
                    return Ok(measurement);
                }

            },
            MeasurementSaturation::Full => {
                // lower the gain
                if let Some(lower_gain) = gain.less_gain() {
                    println!("lowering the gain to: {:?}", lower_gain);

                    lower_gain.apply_to_device(device)?;
                    self.gain = Some(lower_gain);

                    // try to throw away the current measurement
                    (self.get_measurement)(device);

                    return Err(GainControllerError::InProgress);
                } else {
                    // we're as low as it goes
                    return Ok(measurement);
                }
            }
        }
    }
}