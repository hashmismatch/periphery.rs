use periphery_core::prelude::v1::*;
use periphery_core::*;

use std::time::*;
use std::thread;

use ::gesture_detection::*;

use sensor::*;
use registers::*;



pub struct Apds9960Integration<S, B> where S: SystemApi, B: DeviceRegisterBus  {
    sensor: Apds9960<S, B>,
    max_gesture_points: usize,
    gesture_buffer: Vec<GestureSensorPoint>,
    sample_ambient_light_every_ms: usize,
    last_ambient_light_sample: Instant,
    mode: ApdsCurrentMode,
    als_gain_controller: Box<GainController<RGBCData, Apds9960<S, B>, ApdsGain>>
}

impl<S: 'static, B: 'static> Apds9960Integration<S, B> where S: SystemApi, B: DeviceRegisterBus  {
    pub fn new(sensor: Apds9960<S, B>) -> Result<Self, PeripheryError> {
        let now = Instant::now();

        let als_gain_controller = GainControllerImpl::new(&sensor, 
            |apds| {
                apds.get_ambient_light_raw()
            },
            |gain: &ApdsGain, channels| {
                let (min_value, max_value) = {
                    let max_value = gain.sensor_settings.adc_integration_time.get_max_value();
                    
                    let b = max_value / 10;
                    (b, max_value - b)
                };
                
                if channels.clear < min_value &&
                   channels.red < min_value &&
                   channels.green < min_value &&
                   channels.blue < min_value
                {
                    MeasurementSaturation::Empty
                } else if 
                    channels.clear > max_value &&
                    channels.red > max_value &&
                    channels.green > max_value &&
                    channels.blue > max_value
                {
                    MeasurementSaturation::Full
                } else {
                    MeasurementSaturation::Valid
                }
            }        
        );

        sensor.init()?;

        let s = Apds9960Integration {
            sensor: sensor,
            max_gesture_points: 500,
            gesture_buffer: vec![],
            sample_ambient_light_every_ms: 500,
            last_ambient_light_sample: now,
            mode: ApdsCurrentMode::Wait,
            als_gain_controller: Box::new(als_gain_controller)
        };

        s.init()?;
        //println!("initialized");

        Ok(s)
    }

    pub fn get_mode(&self) -> ApdsCurrentMode {
        self.mode
    }

    fn init(&self) -> Result<(), PeripheryError> {
        let r = self.sensor.registers();

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
        //r.wait_time().write(&50)?;
        r.proximity_pulse_count().write(&ProximityPulseCount {
            proximity_pulse_length: 3.into(),
            proximity_pulse_count: 9.into()
        })?;
        r.proximity_offset_up_right().write(&0)?;
        r.proximity_offset_down_left().write(&0)?;

        r.config1().write(&Config1 {
            wait_long: false
        })?;
        r.control1().write(&ControlRegister1 {
            led_drive_strength: 0.into(),
            proximity_gain_control: ProximityGain::Times8,
            als_and_color_gain: AlsGain::Times4
        })?;

        
        r.proximity_interrupt_low_threshold().write(&0)?;
        r.proximity_interrupt_high_threshold().write(&255)?;        
        r.persistence().write(&Persistence {
            proximity_interrupt_persistence: 2.into(),
            als_interrupt_persistence: 0.into(),
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

        r.gesture_proximity_enter_threshold().write(&50)?;
        r.gesture_proximity_exit_threshold().write(&20)?;

        r.gesture_config1().write(&GestureConfig1 {
            gesture_fifo_threshold: 2.into(),
            gesture_exit_mask: 0.into(),
            gesture_exit_persistence: 2.into()
        })?;

        r.gesture_config2().write(&GestureConfig2 {
            gesture_gain: GestureGain::Times4,
            gesture_led_drive_strength: GestureLedDrive::Current100mA,
            gesture_wait_time: 0.into()
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
        
        let enable_register = EnableRegister {
            gesture_enable: true,
            proximity_interrupt_enable: true,
            als_interrupt_enable: true,
            wait_enable: true,
            proximity_detect_enable: true,
            als_enable: true,
            power_on: true
        };
        
        r.enable().write(&enable_register)?;

        Ok(())
    }

    pub fn tick(&mut self) -> Result<ApdsMeasurement, PeripheryError> {
        
        let mut measurement = ApdsMeasurement::InProcessing;

        let current_mode = {
            let status = self.sensor.registers().status().read()?;
            let gesture_config4 = self.sensor.registers().gesture_config4().read()?;
            
            if status.gesture_interrupt || gesture_config4.gesture_mode {
                if let Ok(gestures) = self.sensor.get_gestures() {
                    //println!("gesture ({}) raw: {:?}", gestures.len(), gestures);
                    for gesture in gestures {
                        let p = GestureSensorPoint {
                            milliseconds: 0.0,
                            up: (gesture.up as f32),
                            down: (gesture.down as f32),
                            left: (gesture.left as f32),
                            right: (gesture.right as f32)
                        };
                        self.gesture_buffer.push(p);
                    }

                    while self.gesture_buffer.len() > self.max_gesture_points {
                        self.gesture_buffer.remove(0);
                    }
                }

                ApdsCurrentMode::Gesture
            } else {
                if status.proximity_interrupt {
                    let proximity = self.sensor.registers().proximity_data().read()?;
                    //println!("proximity = {}", proximity);
                    self.sensor.registers().proximity_interrupt_clear().write(&0)?;
                    ApdsCurrentMode::Proximity
                } else if status.als_interrupt {
                    if self.last_ambient_light_sample.elapsed() > Duration::from_millis(self.sample_ambient_light_every_ms as u64) {
                        if let Ok(als_raw) = self.als_gain_controller.tick(&self.sensor) {
                            self.last_ambient_light_sample = Instant::now();

                            if let &Some(gain) = self.als_gain_controller.get_current_gain() {
                                if let Ok(al) = gain.sensor_settings.sensor_to_physical(als_raw) {
                                    /*
                                    if debug_lx_at.elapsed() > Duration::from_millis(1000) {
                                        println!("ambient light level: {:?}", al);
                                        debug_lx_at = Instant::now();
                                    }
                                    */

                                    //println!("als raw: {:?}, gain: {:?}", als_raw, gain);

                                    measurement = ApdsMeasurement::AmbientLight(al);
                                }
                            }
                        }

                    }
                    
                    ApdsCurrentMode::AmbientLight
                } else {
                    ApdsCurrentMode::Wait
                }
            }

        };
                    
        if current_mode == ApdsCurrentMode::Gesture || self.mode == ApdsCurrentMode::Gesture {
            let mut detector = GestureDetector::new();
            let gestures = detector.detect(&self.gesture_buffer);

            if let Some(gesture) = gestures.first() {
                measurement = ApdsMeasurement::Gesture(*gesture);
            }
            /*
            for gesture in &gestures {
                println!("detected gesture: {:?}", gesture);
            }
            */

            if gestures.len() > 0 {
                self.gesture_buffer.clear();
            }
        }

        match (self.mode, current_mode) {
            (ApdsCurrentMode::Gesture, ApdsCurrentMode::Gesture) => (),
            (ApdsCurrentMode::Gesture, _) => {
                // exiting the gesture mode

                let n = self.gesture_buffer.len();
                if n > 10 {
                    println!("samples: {}", n);
                    println!("whole buffer: {:?}", self.gesture_buffer);
                }
                self.gesture_buffer.clear();
            },
            (_, _) => ()
        }

        if current_mode != self.mode {
            //println!("mode: {:?} => {:?}", mode, current_mode);
            self.mode = current_mode;
        }
                    
        
        Ok(measurement)
    }

    pub fn get_wait_time_ms(&self) -> usize {
        match self.mode {
            ApdsCurrentMode::Wait | ApdsCurrentMode::AmbientLight => 50,
            ApdsCurrentMode::Proximity | ApdsCurrentMode::Gesture => 10
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ApdsCurrentMode {
    Wait,
    AmbientLight,
    Proximity,
    Gesture
} 

#[derive(Clone, Debug)]
pub enum ApdsMeasurement {
    InProcessing,
    AmbientLight(Illuminance),
    Gesture(Gesture)
}

#[derive(Copy, Clone, Debug)]
pub struct ApdsGain {
    sensor_settings: AmbientLightSettings
}

use ::autogain::*;

impl<S: 'static, B: 'static> Gain<Apds9960<S, B>> for ApdsGain where S: SystemApi, B: DeviceRegisterBus  {
    fn retrieve_from_device(device: &Apds9960<S, B>) -> Result<Self, PeripheryError> {
        let settings = device.get_ambient_light_settings()?;
        
        Ok(ApdsGain {
            sensor_settings: settings
        })
    }

    fn apply_to_device(&self, device: &Apds9960<S, B>) -> Result<(), PeripheryError> {
        device.registers().control1().modify(|c| {
            c.als_and_color_gain = self.sensor_settings.als_and_color_gain;
        })?;
        device.registers().adc_integration_time().write(&self.sensor_settings.adc_integration_time)?;
        Ok(())
    }

    fn default_gain() -> Self {
        ApdsGain {
            sensor_settings: AmbientLightSettings {
                als_and_color_gain: AlsGain::Times4,
                adc_integration_time: AdcIntegrationTime::new_from_cycles(10)
            }
        }
    }

    fn less_gain(&self) -> Option<Self> {
        let g = match self.sensor_settings.als_and_color_gain {
            AlsGain::Times1 => None,
            AlsGain::Times4 => Some(AlsGain::Times1),
            AlsGain::Times16 => Some(AlsGain::Times4),
            AlsGain::Times64 => Some(AlsGain::Times16)
        };

        g.map(|g| {
            let mut settings = self.sensor_settings;
            settings.als_and_color_gain = g;
            ApdsGain { sensor_settings: settings }
        })
    }

    fn more_gain(&self) -> Option<Self> {
        let g = match self.sensor_settings.als_and_color_gain {
            AlsGain::Times1 => Some(AlsGain::Times4),
            AlsGain::Times4 => Some(AlsGain::Times16),
            AlsGain::Times16 => Some(AlsGain::Times64),
            AlsGain::Times64 => None
        };

        g.map(|g| {
            let mut settings = self.sensor_settings;
            settings.als_and_color_gain = g;
            ApdsGain { sensor_settings: settings }
        })
    }
}
