use periphery_core::prelude::v1::*;
use periphery_core::*;
use periphery_core::terminal_cli::*;

use super::cli::*;
use super::orientation::*;

pub fn detect_gyro_axis<'a, S: SystemApi>(axis: GyroAxis, expected_orientation: AxisOrientation, sensor: &AngularSpeed3Sensor, ctx: &mut CommandContext<'a>, system: &S)
    -> Option<(AxisOrientation, GyroAxis)>
{
    ctx.get_terminal().print_line("Calibrating the gyro's zero offset.");
    system.get_sleep().unwrap().sleep_ms(1000);

    let gyro_offset = {
        let n = 100;
        let mut m = GyroOffsetMeasurement::new(n);

        loop {
            match sensor.get_angular_speed_3() {
                Ok(a) => {
                    match m.sample(a) {
                        GyroOffsetMeasurementStatus::MoreSamplesRequired => {
                            system.get_sleep().unwrap().sleep_ms(1);
                        },
                        GyroOffsetMeasurementStatus::Done => {
                            break;
                        }
                    }
                },
                Err(e) => {
                    ctx.get_terminal().print_line(&format!("Error retrieving gyro measurement: {:?}", e));
                    return None;
                }
            }
        }

        m.to_filter()
    };

    ctx.get_terminal().print_line(&format!("Static offset: {:?}", gyro_offset.get_offset()));

    ctx.get_terminal().print_line("");

    help(Detection::Gyro(axis), ctx);

    ctx.get_terminal().print_line("");
    
    delay_before_move(ctx, system);

    let angle_required = 18.0;

    let mut angle = (0.0, 0.0, 0.0);

    let mut detected_axis = None;

    let seconds = 5;
    let sampling_rate_hz = 100; 
    
    for i in 0..(seconds * sampling_rate_hz) {
        match sensor.get_angular_speed_3() {
            Ok(a) => {
                let a = gyro_offset.apply(a);

                angle.0 += a.x.get_degrees_per_second() / sampling_rate_hz as f32;
                angle.1 += a.y.get_degrees_per_second() / sampling_rate_hz as f32;
                angle.2 += a.z.get_degrees_per_second() / sampling_rate_hz as f32;

                if angle.0.abs() > angle_required {
                    detected_axis = Some((angle.0, GyroAxis::Roll));
                } else if angle.1.abs() > angle_required {
                    detected_axis = Some((angle.1, GyroAxis::Pitch));
                } else if angle.2.abs() > angle_required {
                    detected_axis = Some((angle.2, GyroAxis::Yaw));
                }

                if detected_axis.is_some() {
                    break;
                } 

                system.get_sleep().unwrap().sleep_ms(1000 / sampling_rate_hz);
            },
            Err(e) => {
                ctx.get_terminal().print_line(&format!("Error retrieving gyro measurement: {:?}", e));
                return None;
            }
        }
    }

    ctx.get_terminal().print_line("");

    if let Some((n, detected_axis)) = detected_axis {
        let n = if expected_orientation == AxisOrientation::Negative { -n } else { n };
        let p = if n >= 0.0 { "+" } else { "-" };
        
        let raw_axis: RawSensorAxis = detected_axis.into();

        ctx.get_terminal().print_line(&format!("Measured movement [degrees]: {:?}", angle));
        ctx.get_terminal().print_line(&format!("Detected axis: Gyro {:?} = {}{}", axis, p, raw_axis));        

        let orientation = if n >= 0.0 { AxisOrientation::Positive } else { AxisOrientation::Negative };

        Some((orientation, detected_axis))
    } else {
        ctx.get_terminal().print_line(&format!("No movement detected. Final gyro angles [degrees]: {:?}", angle));

        None
    }
}



pub struct GyroOffsetMeasurement {
    m: AngularSpeed3,
    n: usize,
    i: usize
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GyroOffsetMeasurementStatus {
    MoreSamplesRequired,
    Done
}

impl GyroOffsetMeasurement {
    pub fn new(n: usize) -> GyroOffsetMeasurement {
        GyroOffsetMeasurement {
            m: AngularSpeed3 {
                x: AngularSpeed::from_degrees_per_second(0.0),
                y: AngularSpeed::from_degrees_per_second(0.0),
                z: AngularSpeed::from_degrees_per_second(0.0)
            },

            n: n,
            i: 0
        }
    }

    pub fn sample(&mut self, measurement: AngularSpeed3) -> GyroOffsetMeasurementStatus {
        if self.i >= self.n {
            return GyroOffsetMeasurementStatus::Done;
        }

        self.m = AngularSpeed3 {
            x: AngularSpeed::from_degrees_per_second(self.m.x.get_degrees_per_second() + measurement.x.get_degrees_per_second()),
            y: AngularSpeed::from_degrees_per_second(self.m.y.get_degrees_per_second() + measurement.y.get_degrees_per_second()),
            z: AngularSpeed::from_degrees_per_second(self.m.z.get_degrees_per_second() + measurement.z.get_degrees_per_second())
        };
        self.i += 1;
        
        if self.i >= self.n {
            GyroOffsetMeasurementStatus::Done
        } else {
            GyroOffsetMeasurementStatus::MoreSamplesRequired
        }
    }

    pub fn to_filter(&self) -> GyroOffsetStaticCompensation {
        GyroOffsetStaticCompensation {
            offset: AngularSpeed3 {
                x: AngularSpeed::from_degrees_per_second(self.m.x.get_degrees_per_second() / (self.i as f32)),
                y: AngularSpeed::from_degrees_per_second(self.m.y.get_degrees_per_second() / (self.i as f32)),
                z: AngularSpeed::from_degrees_per_second(self.m.z.get_degrees_per_second() / (self.i as f32))
            }
        }
    }
}

pub struct GyroOffsetStaticCompensation {
    offset: AngularSpeed3
}

impl GyroOffsetStaticCompensation {
    pub fn new(offset: AngularSpeed3) -> GyroOffsetStaticCompensation {
        GyroOffsetStaticCompensation {
            offset: offset
        }
    }

    pub fn get_offset(&self) -> &AngularSpeed3 {
        &self.offset
    }

    pub fn apply(&self, measurement: AngularSpeed3) -> AngularSpeed3 {
        AngularSpeed3 {
            x: AngularSpeed::from_degrees_per_second(measurement.x.get_degrees_per_second() - self.offset.x.get_degrees_per_second()),
            y: AngularSpeed::from_degrees_per_second(measurement.y.get_degrees_per_second() - self.offset.y.get_degrees_per_second()),
            z: AngularSpeed::from_degrees_per_second(measurement.z.get_degrees_per_second() - self.offset.z.get_degrees_per_second())
        }
    }
}