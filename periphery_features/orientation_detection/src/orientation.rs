use periphery_core::prelude::v1::*;
use periphery_core::*;
use periphery_core::terminal_cli::*;

use super::utils::*;

pub fn show_sensor_axis_rotation(term: &mut CharacterTerminalWriter, axes: [(AxisOrientation, RawSensorAxis); 3]) {
    term.print_line("Final sensor rotation");

    let mut print_axis = |axis_left, axis_measured: (AxisOrientation, RawSensorAxis)| {
        term.print_line(&format!("{} = {}{}", axis_left, axis_measured.0, axis_measured.1));
    };

    print_axis(RawSensorAxis::X, axes[0]);
    print_axis(RawSensorAxis::Y, axes[1]);
    print_axis(RawSensorAxis::Z, axes[2]);
}

pub fn delay_before_move<'a, S: SystemApi>(ctx: &mut CommandContext<'a>, system: &S) {
    let n = 5;
    for i in 0..5 {
        ctx.get_terminal().print_str(&format!("{} ... ", n-i));
        system.sleep_ms(1000);
    }

    ctx.get_terminal().print_str(&format!("Move!"));

    ctx.get_terminal().print_line("");
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Detection {
    Acc(AccAxis),
    Gyro(GyroAxis)
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum RawSensorAxis {
    X, Y, Z
}

impl RawSensorAxis {
    pub fn to_index(&self) -> usize {
        match *self {
            RawSensorAxis::X => 0,
            RawSensorAxis::Y => 1,
            RawSensorAxis::Z => 2
        }
    }
}

impl Display for RawSensorAxis {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            RawSensorAxis::X => f.write_str("X"),
            RawSensorAxis::Y => f.write_str("Y"),
            RawSensorAxis::Z => f.write_str("Z")
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum GyroAxis {
    Roll, Pitch, Yaw
}

impl Into<RawSensorAxis> for GyroAxis {
    fn into(self) -> RawSensorAxis {
        match self {
            GyroAxis::Roll => RawSensorAxis::X,
            GyroAxis::Pitch => RawSensorAxis::Y,
            GyroAxis::Yaw => RawSensorAxis::Z
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AccAxis {
    X, Y, Z
}

impl Into<RawSensorAxis> for AccAxis {
    fn into(self) -> RawSensorAxis {
        match self {
            AccAxis::X => RawSensorAxis::X,
            AccAxis::Y => RawSensorAxis::Y,
            AccAxis::Z => RawSensorAxis::Z
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AxisOrientation {
    Positive,
    Negative
}

impl Default for AxisOrientation {
    fn default() -> Self {
        AxisOrientation::Positive
    }
}

impl AxisOrientation {
    pub fn apply(&self, val: f32) -> f32 {
        if *self == AxisOrientation::Negative { -val } else { val }
    }
}

impl Display for AxisOrientation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            AxisOrientation::Positive => f.write_str("+"),
            AxisOrientation::Negative => f.write_str("-")
        }
    }
}



#[derive(Copy, Clone, Debug, PartialEq)]
pub struct SensorAxis(pub AxisOrientation, pub RawSensorAxis);

extern crate permutohedron;

pub fn sensor_3_axis_all_configurations() -> Vec<[SensorAxis; 3]> {
    let mut ret = vec![];

    let axes_orientations = [AxisOrientation::Positive, AxisOrientation::Negative];
    let mut permutations = PermutationsWithRepetitions::new(&axes_orientations, 3);
    let axes_orientations_permutations: Vec<_> = permutations.collect();
    
    {
        let mut axes_indices = [RawSensorAxis::X, RawSensorAxis::Y, RawSensorAxis::Z];
        let axes_orientations = [AxisOrientation::Positive, AxisOrientation::Negative];

        let mut axes = permutohedron::Heap::new(&mut axes_indices);
        loop {
            match axes.next_permutation() {
                Some(a) => {

                    for signs in &axes_orientations_permutations {
                        ret.push([
                            SensorAxis(signs[0], a[0]),
                            SensorAxis(signs[1], a[1]),
                            SensorAxis(signs[2], a[2])
                        ])
                    }
                    
                    
                }
                None => { break; }
            }
        }
    }

    ret
}


