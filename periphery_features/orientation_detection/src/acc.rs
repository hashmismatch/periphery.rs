use periphery_core::prelude::v1::*;
use periphery_core::*;
use periphery_core::terminal_cli::*;

use cli::*;
use orientation::*;

pub fn detect_acc_axis<'a, S: SystemApi>(axis: AccAxis, expected_orientation: AxisOrientation, sensor: &Acceleration3Sensor, ctx: &mut CommandContext<'a>, system: &S)
    -> Option<(AxisOrientation, AccAxis)>
{
    help(Detection::Acc(axis), ctx);

    ctx.get_terminal().print_line("");
    
    let n = 5;
    for i in 0..5 {
        ctx.get_terminal().print_str(&format!("{} ... ", n-i));
        system.get_sleep().unwrap().sleep_ms(1000);
    }

    ctx.get_terminal().print_str(&format!("Move!"));

    ctx.get_terminal().print_line("");

    delay_before_move(ctx, system);

    let mut detected_axis = None;
    let measured = match sensor.get_acceleration_3() {
        Ok(a) => {

            let x = a.x.get_g_force();
            let y = a.y.get_g_force();
            let z = a.z.get_g_force();

            let mut m = 0.0;

            if x.abs() > m {
                m = x.abs();
                detected_axis = Some((x, AccAxis::X));
            }
            if y.abs() > m {
                m = y.abs();
                detected_axis = Some((y, AccAxis::Y));
            }
            if z.abs() > m {
                m = z.abs();
                detected_axis = Some((z, AccAxis::Z));
            }

            a

        },
        Err(e) => {
            ctx.get_terminal().print_line(&format!("Error retrieving acc measurement: {:?}", e));
            return None;
        }
    };

    ctx.get_terminal().print_line("");

    if let Some((n, detected_axis)) = detected_axis {
        let n = if expected_orientation == AxisOrientation::Negative { -n } else { n };
        let p = if n >= 0.0 { "+" } else { "-" };
        let raw_axis: RawSensorAxis = detected_axis.into();

        ctx.get_terminal().print_line(&format!("Measurement: {:?}", measured));
        ctx.get_terminal().print_line(&format!("Detected axis: Acc {:?} = {}{}", axis, p, raw_axis));

        let orientation = if n >= 0.0 { AxisOrientation::Positive } else { AxisOrientation::Negative };
        Some((orientation, detected_axis))
    } else {
        ctx.get_terminal().print_line(&format!("No axis detected."));
        None
    }
}
