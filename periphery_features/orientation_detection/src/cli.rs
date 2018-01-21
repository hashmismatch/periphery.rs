use periphery_core::prelude::v1::*;
use periphery_core::*;
use periphery_core::terminal_cli::*;

use super::acc::*;
use super::gyro::*;
use super::mag::*;
use super::orientation::*;

pub fn sensor_orientation_cli<'a, S: SystemApi>(sensor: &Device, exec: &mut CliExecutor<'a>, system: &S) -> Result<(), PeripheryError> {

    system.get_sleep()?;
    
    if let Some(acc) = sensor.get_acceleration_3_sensor() {
        let axis_x = |ctx: &mut CommandContext| { detect_acc_axis(AccAxis::X, AxisOrientation::Positive, acc, ctx, system) };
        let axis_y = |ctx: &mut CommandContext| { detect_acc_axis(AccAxis::Y, AxisOrientation::Positive, acc, ctx, system) };
        let axis_z = |ctx: &mut CommandContext| { detect_acc_axis(AccAxis::Z, AxisOrientation::Positive, acc, ctx, system) };

    
        let cmd = format!("{}/orientation/acc/x", sensor.id());
        if let Some(mut ctx) = exec.command(&cmd) {
            axis_x(&mut ctx);
        }
    
        
        let cmd = format!("{}/orientation/acc/y", sensor.id());
        if let Some(mut ctx) = exec.command(&cmd) {
            axis_y(&mut ctx);
        }

        let cmd = format!("{}/orientation/acc/z", sensor.id());
        if let Some(mut ctx) = exec.command(&cmd) {
            axis_z(&mut ctx);
        }

        let cmd = format!("{}/orientation/acc/all", sensor.id());
        if let Some(mut ctx) = exec.command(&cmd) {
            let x = axis_x(&mut ctx);
            let y = axis_y(&mut ctx);
            let z = axis_z(&mut ctx);

            ctx.get_terminal().print_line("");
            ctx.get_terminal().print_line("");

            match (x, y, z) {
                (Some(x), Some(y), Some(z)) => {
                    show_sensor_axis_rotation(ctx.get_terminal(), [
                        (x.0, x.1.into()),
                        (y.0, y.1.into()),
                        (z.0, z.1.into()),
                    ]);
                },
                _ => ()
            }            
        }        
    }

    if let Some(gyro) = sensor.get_angular_speed_3_sensor() {
        let axis_roll = |ctx: &mut CommandContext| { detect_gyro_axis(GyroAxis::Roll, AxisOrientation::Positive, gyro, ctx, system) };
        let axis_pitch = |ctx: &mut CommandContext| { detect_gyro_axis(GyroAxis::Pitch, AxisOrientation::Negative, gyro, ctx, system) };
        let axis_yaw = |ctx: &mut CommandContext| { detect_gyro_axis(GyroAxis::Yaw, AxisOrientation::Negative, gyro, ctx, system) };



        let cmd = format!("{}/orientation/gyro/roll", sensor.id());
        if let Some(mut ctx) = exec.command(&cmd) {
            axis_roll(&mut ctx);
        }

        let cmd = format!("{}/orientation/gyro/pitch", sensor.id());
        if let Some(mut ctx) = exec.command(&cmd) {
            axis_pitch(&mut ctx);
        }

        let cmd = format!("{}/orientation/gyro/yaw", sensor.id());
        if let Some(mut ctx) = exec.command(&cmd) {
            axis_yaw(&mut ctx);
        }

        let cmd = format!("{}/orientation/gyro/all", sensor.id());
        if let Some(mut ctx) = exec.command(&cmd) {
            let x = axis_roll(&mut ctx);
            ctx.get_terminal().print_line("Lie the body down on the ground.");
            system.get_sleep().unwrap().sleep_ms(5000);

            let y = axis_pitch(&mut ctx);
            ctx.get_terminal().print_line("Lie the body down on the ground.");
            system.get_sleep().unwrap().sleep_ms(5000);

            let z = axis_yaw(&mut ctx);            

            ctx.get_terminal().print_line("");
            ctx.get_terminal().print_line("");

            match (x, y, z) {
                (Some(x), Some(y), Some(z)) => {
                    show_sensor_axis_rotation(ctx.get_terminal(), [
                        (x.0, x.1.into()),
                        (y.0, y.1.into()),
                        (z.0, z.1.into()),
                    ]);
                },
                _ => ()
            }
        }
    }

    if let Some(mag) = sensor.get_magnetic_field_3_sensor() {
        let axis_xy = |ctx: &mut CommandContext| { detect_mag_xy_axis(mag, ctx, system) };
        let axis_yz = |ctx: &mut CommandContext| { detect_mag_yz_axis(mag, ctx, system) };

        let cmd = format!("{}/orientation/mag/xy", sensor.id());
        if let Some(mut ctx) = exec.command(&cmd) {
            axis_xy(&mut ctx);
        }

        let cmd = format!("{}/orientation/mag/yz", sensor.id());
        if let Some(mut ctx) = exec.command(&cmd) {
            axis_yz(&mut ctx);
        }
    }

    Ok(())
}


pub fn help<'a>(detect: Detection, ctx: &mut CommandContext<'a>) {    
    ctx.get_terminal().print_line("Detecting orientation. Reference body coordinate system: NED.");
    ctx.get_terminal().print_line("https://developer.dji.com/onboard-sdk/documentation/introduction/things-to-know.html");    
    ctx.get_terminal().print_line("Observe the body from the back and look down onto it.");

    let txt = match detect {
        Detection::Acc(AccAxis::X) => "X axis. Stand the body on its back side and hold it there.",
        Detection::Acc(AccAxis::Y) => "Y axis. Stand the body on its right side and hold it there.",
        Detection::Acc(AccAxis::Z) => "Z axis. Keep the body still on the ground.",

        Detection::Gyro(GyroAxis::Roll) => "Roll. Pull the left side upwards.",
        Detection::Gyro(GyroAxis::Pitch) => "Pitch. Pull the front upwards.",
        Detection::Gyro(GyroAxis::Yaw) => "Yaw. Rotate the body clockwise."
    };

    ctx.get_terminal().print_line(&txt);
    ctx.get_terminal().print_line("Move the body when instructed.");
}
