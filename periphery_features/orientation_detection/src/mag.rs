use periphery_core::prelude::v1::*;
use periphery_core::*;
use periphery_core::terminal_cli::*;

use super::orientation::*;

fn sample_mag<'a, S: SystemApi>(sensor: &MagneticField3Sensor, ctx: &mut CommandContext<'a>, system: &S) -> Result<MagneticField3, PeripheryError> {
    let seconds = 10;
    let sampling_rate_hz = 5;

    let mut ret = Default::default();

    for i in 0..(seconds * sampling_rate_hz) {

        let m = sensor.get_magnetic_field_3()?;
        ctx.get_terminal().print_line(&format!("{:?}", m));

        ret = m;

        system.get_sleep().unwrap().sleep_ms(1000 / sampling_rate_hz);        
    }

    Ok(ret)
}

pub fn detect_mag_xy_axis<'a, S: SystemApi>(sensor: &MagneticField3Sensor, ctx: &mut CommandContext<'a>, system: &S)
    -> Result<((AxisOrientation, RawSensorAxis), (AxisOrientation, RawSensorAxis)), PeripheryError>
{
    ctx.get_terminal().print_line("Detecting mag XY orientation.");
    ctx.get_terminal().print_line("During movement, the sensor's reading will be shown. One value should be as close as possible to zero.");
    ctx.get_terminal().print_line("Step 1: point the front of the body directly towards north. Move when instructed and hold until the end of measurement.");

    ctx.get_terminal().print_line("");
    
    delay_before_move(ctx, system);
    let north = sample_mag(sensor, ctx, system)?;

    ctx.get_terminal().print_line("");
    ctx.get_terminal().print_line("");

    ctx.get_terminal().print_line("During movement, the sensor's reading will be shown. One value should be as close as possible to zero.");
    ctx.get_terminal().print_line("Step 2: point the front of the body directly towards east (90 degrees clockwise from north). Move when instructed and hold until the end of measurement.");

    delay_before_move(ctx, system);
    let east = sample_mag(sensor, ctx, system)?;

    ctx.get_terminal().print_line("");
    ctx.get_terminal().print_line("");

    ctx.get_terminal().print_line("During movement, the sensor's reading will be shown. One value should be as close as possible to zero.");
    ctx.get_terminal().print_line("Step 3: point the front of the body directly towards south. Move when instructed and hold until the end of measurement.");

    delay_before_move(ctx, system);

    let south = sample_mag(sensor, ctx, system)?;

    ctx.get_terminal().print_line("");
    ctx.get_terminal().print_line("");

    ctx.get_terminal().print_line("Raw readings");
    ctx.get_terminal().print_line(&format!("North: {:?}, East: {:?}, South: {:?}", north, east, south));

    ctx.get_terminal().print_line("");


    if let Ok(xy) = mag_xy_orientation(north, east, south) {
        ctx.get_terminal().print_line(&format!("X: {:?}", xy.x));
        ctx.get_terminal().print_line(&format!("Y: {:?}", xy.y));
    } else {
        ctx.get_terminal().print_line("Failed to detect the correct sensor rotations.");
    }


    Err(PeripheryError::Unknown)
}


pub fn detect_mag_yz_axis<'a, S: SystemApi>(sensor: &MagneticField3Sensor, ctx: &mut CommandContext<'a>, system: &S)
    -> Result<((AxisOrientation, RawSensorAxis), (AxisOrientation, RawSensorAxis)), PeripheryError>
{
    ctx.get_terminal().print_line("Detecting mag YZ orientation.");
    ctx.get_terminal().print_line("During movement, the sensor's reading will be shown. One value should be as close as possible to zero.");
    ctx.get_terminal().print_line("Step 1: point the front up towards the sky and the right side towards north. Move when instructed and hold until the end of measurement.");

    ctx.get_terminal().print_line("");
    
    delay_before_move(ctx, system);
    let north = sample_mag(sensor, ctx, system)?;

    ctx.get_terminal().print_line("");
    ctx.get_terminal().print_line("");

    ctx.get_terminal().print_line("During movement, the sensor's reading will be shown. One value should be as close as possible to zero.");
    ctx.get_terminal().print_line("Step 2: point the front up towards the sky and the right side directly towards east (90 degrees clockwise from north). Move when instructed and hold until the end of measurement.");

    delay_before_move(ctx, system);
    let east = sample_mag(sensor, ctx, system)?;

    ctx.get_terminal().print_line("");
    ctx.get_terminal().print_line("");

    ctx.get_terminal().print_line("During movement, the sensor's reading will be shown. One value should be as close as possible to zero.");
    ctx.get_terminal().print_line("Step 3: point the front up towards the sky and the right side towards south. Move when instructed and hold until the end of measurement.");

    delay_before_move(ctx, system);

    let south = sample_mag(sensor, ctx, system)?;

    ctx.get_terminal().print_line("");
    ctx.get_terminal().print_line("");

    ctx.get_terminal().print_line("Raw readings");
    ctx.get_terminal().print_line(&format!("North: {:?}, East: {:?}, South: {:?}", north, east, south));

    ctx.get_terminal().print_line("");


    if let Ok(yz) = mag_yz_orientation(north, east, south) {
        ctx.get_terminal().print_line(&format!("Y: {:?}", yz.y));
        ctx.get_terminal().print_line(&format!("Z: {:?}", yz.z));
    } else {
        ctx.get_terminal().print_line("Failed to detect the correct sensor rotations.");
    }


    Err(PeripheryError::Unknown)
}



fn mag_check_all_orientations<F: Fn(MagneticField3) -> (MagneticFieldStrength, MagneticFieldStrength)>
    (north: MagneticField3, east: MagneticField3, south: MagneticField3, f: F) -> Result<[SensorAxis; 3], PeripheryError>
{
    let configs = sensor_3_axis_all_configurations();

    let least_error = configs.iter()
        .map(|x| { (mag_orientation_degrees_error(north, east, south, *x, &f), x) })
        .filter(|x| x.0 <= 50.0)
        .min_by_key(|x| x.0 as i32);

    let least_error = least_error.ok_or(PeripheryError::DataNotAvailable)?;
    Ok(*(least_error.1))
}


#[derive(Copy, Clone, Debug)]
struct MagXySensor { x: SensorAxis, y: SensorAxis }

fn mag_xy_orientation(north: MagneticField3, east: MagneticField3, south: MagneticField3) -> Result<MagXySensor, PeripheryError> {
    let s = mag_check_all_orientations(north, east, south, |m| (m.x, m.y))?;
    
    Ok(MagXySensor {
        x: s[0],
        y: s[1],
    })
}


#[derive(Copy, Clone, Debug)]
struct MagYzSensor { y: SensorAxis, z: SensorAxis }

fn mag_yz_orientation(north: MagneticField3, east: MagneticField3, south: MagneticField3) -> Result<MagYzSensor, PeripheryError> {
    let s = mag_check_all_orientations(north, east, south, |m| {
        let y = -m.y.get_gauss();
        let z = -m.z.get_gauss();
        
        (MagneticFieldStrength::from_gauss(y), MagneticFieldStrength::from_gauss(z))
    })?;
    
    Ok(MagYzSensor {
        y: s[1],
        z: s[2],
    })
}

fn mag_orientation_degrees_error<F: Fn(MagneticField3) -> (MagneticFieldStrength, MagneticFieldStrength)>
    (north: MagneticField3, east: MagneticField3, south: MagneticField3, s: [SensorAxis; 3], compass: F) -> f32
{
    let north = apply_mag_rotation(north, s);
    let east = apply_mag_rotation(east, s);
    let south = apply_mag_rotation(south, s);


    let north_degrees = {
        let c = compass(north);
        mag_azimuth_degrees(c.0, c.1)
    };
    let east_degrees = {
        let c = compass(east);
        mag_azimuth_degrees(c.0, c.1)
    };
    let south_degrees = {
        let c = compass(south);
        mag_azimuth_degrees(c.0, c.1)
    };

    let a = compass_difference(north_degrees, 0.0);
    let b = compass_difference(east_degrees, 90.0);
    let c = compass_difference(south_degrees, 180.0);

    (a.abs() + b.abs() + c.abs())
}


fn compass_degrees_normalize(mut h: f32) -> f32 {
    while h < 0.0 { h += 360.0; }
    while h >= 360.0 { h -= 360.0; }
    h
}

/// calculate the shortest distance in yaw to get from b => a
fn compass_difference(a: f32, b: f32) -> f32 {
    let mut diff = b - a;

    while diff > 180.0 { diff -= 360.0; }
    while diff <= -180.0 { diff += 360.0; }

    diff
}


fn apply_mag_rotation(m: MagneticField3, s: [SensorAxis; 3]) -> MagneticField3 {
    MagneticField3 {
        x: apply_mag_rotation_axis(m, s[0]),
        y: apply_mag_rotation_axis(m, s[1]),
        z: apply_mag_rotation_axis(m, s[2])
    }
}

fn apply_mag_rotation_axis(m: MagneticField3, s: SensorAxis) -> MagneticFieldStrength {
    let v = m[s.1.to_index()];
    let g = s.0.apply(v.get_gauss());
    MagneticFieldStrength::from_gauss(g)
}



fn mag_azimuth_degrees(x: MagneticFieldStrength, y: MagneticFieldStrength) -> f32 {
    //return (180/math.pi * math.atan2(y, x)) % 360

    ((180.0 / 3.14159265359) * y.get_gauss().atan2(x.get_gauss())) % 360.0
}




#[test]
#[cfg(test)]
fn test_mag_xy_orientation_detection() {
    //North: Magnetic field: X -0.0009174312 Gs, Y -0.2853211 Gs, Z -0.3440367 Gs,
    // East: Magnetic field: X 0.2853211 Gs, Y -0.05321101 Gs, Z -0.3825688 Gs,
    // South: Magnetic field: X -0.0009174312 Gs, Y 0.18990825 Gs, Z -0.31559634 Gs

    // expected result: X = -Y, Y: +X

    let north = MagneticField3 {
        x: MagneticFieldStrength::from_gauss(-0.0009174312),
        y: MagneticFieldStrength::from_gauss(-0.2853211),
        z: MagneticFieldStrength::from_gauss(-0.3440367)
    };

    let east = MagneticField3 {
        x: MagneticFieldStrength::from_gauss(0.2853211),
        y: MagneticFieldStrength::from_gauss(-0.05321101),
        z: MagneticFieldStrength::from_gauss(-0.3825688)
    };

    let south = MagneticField3 {
        x: MagneticFieldStrength::from_gauss(-0.0009174312),
        y: MagneticFieldStrength::from_gauss(0.18990825),
        z: MagneticFieldStrength::from_gauss(-0.31559634)
    };

    let spracing_f3_mag = mag_xy_orientation(north, east, south).unwrap();

    assert_eq!(spracing_f3_mag.x, SensorAxis(AxisOrientation::Negative, RawSensorAxis::Y));
    assert_eq!(spracing_f3_mag.y, SensorAxis(AxisOrientation::Positive, RawSensorAxis::X));
}

#[test]
#[cfg(test)]
fn test_mag_yz_orientation_detection() {
    //North: Magnetic field: X -0.12660551 Gs, Y 0.3972477 Gs, Z 0.0045871558 Gs,
    //East: Magnetic field: X 0.020183487 Gs, Y 0.3733945 Gs, Z -0.17889908 Gs,
    //South: Magnetic field: X 0.25321102 Gs, Y 0.3733945 Gs, Z 0.0018348624 Gs

    // expected result: Y = +X, Z = +Z

    let north = MagneticField3 {
        x: MagneticFieldStrength::from_gauss(-0.12660551),
        y: MagneticFieldStrength::from_gauss(0.3972477),
        z: MagneticFieldStrength::from_gauss(0.0045871558)
    };

    let east = MagneticField3 {
        x: MagneticFieldStrength::from_gauss(0.020183487),
        y: MagneticFieldStrength::from_gauss(0.3733945),
        z: MagneticFieldStrength::from_gauss(-0.17889908)
    };

    let south = MagneticField3 {
        x: MagneticFieldStrength::from_gauss(0.25321102),
        y: MagneticFieldStrength::from_gauss(0.3733945),
        z: MagneticFieldStrength::from_gauss(0.0018348624)
    };

    let spracing_f3_mag = mag_yz_orientation(north, east, south).unwrap();

    assert_eq!(spracing_f3_mag.y, SensorAxis(AxisOrientation::Positive, RawSensorAxis::X));
    assert_eq!(spracing_f3_mag.z, SensorAxis(AxisOrientation::Positive, RawSensorAxis::Z));
    
}
