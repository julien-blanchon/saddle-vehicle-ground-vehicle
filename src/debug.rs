use crate::{GroundVehicleDebugDraw, GroundVehicleWheel, GroundVehicleWheelState};
use bevy::prelude::*;

pub type GroundVehicleDebugDrawRuntime = GroundVehicleDebugDraw;

pub(crate) fn draw_debug_gizmos(
    config: Res<GroundVehicleDebugDrawRuntime>,
    wheels: Query<(&GroundVehicleWheel, &GroundVehicleWheelState)>,
    chassis: Query<&Transform>,
    mut gizmos: Gizmos,
) {
    if !config.enabled {
        return;
    }

    for (wheel, state) in &wheels {
        let Ok(chassis_transform) = chassis.get(wheel.chassis) else {
            continue;
        };
        let origin = chassis_transform.transform_point(wheel.mount_point);
        let up = chassis_transform.rotation * Vec3::Y;
        let droop_end = origin - up * wheel.suspension.max_length();

        if config.draw_suspension {
            let end = if state.grounded {
                state.contact_point
            } else {
                droop_end
            };
            gizmos.line(origin, end, Color::srgb(0.95, 0.95, 0.95));
        }

        if state.grounded && config.draw_contact_normals {
            gizmos.arrow(
                state.contact_point,
                state.contact_point + state.contact_normal.clamp_length_max(1.0),
                Color::srgb(0.25, 0.9, 0.35),
            );
        }

        if state.grounded && config.draw_force_vectors {
            let longitudinal = state.longitudinal_force_newtons * 0.000_25;
            let lateral = state.lateral_force_newtons * 0.000_25;
            let wheel_forward = Quat::from_axis_angle(up, state.steer_angle_rad)
                * (chassis_transform.rotation * Vec3::NEG_Z);
            let wheel_right = wheel_forward
                .cross(state.contact_normal)
                .normalize_or_zero();
            gizmos.arrow(
                state.contact_point,
                state.contact_point + wheel_forward.normalize_or_zero() * longitudinal,
                Color::srgb(0.2, 0.5, 0.95),
            );
            gizmos.arrow(
                state.contact_point,
                state.contact_point + wheel_right * lateral,
                Color::srgb(0.95, 0.35, 0.2),
            );
        }

        if state.grounded && config.draw_slip_vectors {
            let wheel_forward = Quat::from_axis_angle(up, state.steer_angle_rad)
                * (chassis_transform.rotation * Vec3::NEG_Z);
            let wheel_right = wheel_forward
                .cross(state.contact_normal)
                .normalize_or_zero();
            let slip = wheel_forward.normalize_or_zero() * state.longitudinal_speed_mps * 0.08
                + wheel_right * state.lateral_speed_mps * 0.08;
            gizmos.arrow(
                state.contact_point,
                state.contact_point + slip,
                Color::srgb(1.0, 0.85, 0.1),
            );
        }
    }
}
