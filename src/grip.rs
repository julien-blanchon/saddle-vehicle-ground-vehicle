use crate::{
    GroundVehicle, GroundVehicleResolvedControl, GroundVehicleSurface, GroundVehicleWheel,
    GroundVehicleWheelInternal, GroundVehicleWheelState,
};
use avian3d::prelude::*;
use bevy::prelude::*;

pub(crate) fn surface_for_contact(
    contact_entity: Option<Entity>,
    surfaces: &Query<&GroundVehicleSurface>,
    collider_of: &Query<&ColliderOf>,
) -> GroundVehicleSurface {
    let Some(entity) = contact_entity else {
        return GroundVehicleSurface::default();
    };
    if let Ok(surface) = surfaces.get(entity) {
        return *surface;
    }
    if let Ok(owner) = collider_of.get(entity) {
        if let Ok(surface) = surfaces.get(owner.body) {
            return *surface;
        }
    }
    GroundVehicleSurface::default()
}

pub(crate) fn load_scaled_limit(
    base_limit: f32,
    load_newtons: f32,
    nominal_load_newtons: f32,
    sensitivity: f32,
) -> f32 {
    if base_limit <= 0.0 || load_newtons <= 0.0 {
        return 0.0;
    }
    let load_ratio = (load_newtons / nominal_load_newtons.max(1.0)).clamp(0.25, 2.5);
    base_limit * load_ratio.powf(sensitivity.clamp(0.0, 1.0))
}

pub(crate) fn friction_circle_scale(
    longitudinal_force: f32,
    longitudinal_limit: f32,
    lateral_force: f32,
    lateral_limit: f32,
) -> f32 {
    if longitudinal_limit <= 0.0 && lateral_limit <= 0.0 {
        return 0.0;
    }
    let long_ratio = if longitudinal_limit > 0.0 {
        longitudinal_force / longitudinal_limit
    } else {
        0.0
    };
    let lat_ratio = if lateral_limit > 0.0 {
        lateral_force / lateral_limit
    } else {
        0.0
    };
    let magnitude = (long_ratio * long_ratio + lat_ratio * lat_ratio).sqrt();
    if magnitude > 1.0 {
        magnitude.recip()
    } else {
        1.0
    }
}

pub(crate) fn apply_tire_forces(
    time: Res<Time>,
    mut wheels: Query<(
        &GroundVehicleWheel,
        &mut GroundVehicleWheelState,
        &GroundVehicleWheelInternal,
    )>,
    mut chassis: Query<(
        Forces,
        &GroundVehicle,
        &GroundVehicleResolvedControl,
        &Transform,
    )>,
    surfaces: Query<&GroundVehicleSurface>,
    collider_of: Query<&ColliderOf>,
) {
    let dt = time.delta_secs().max(1.0 / 480.0);

    for (wheel, mut state, wheel_internal) in &mut wheels {
        let Ok((mut forces, vehicle, resolved, transform)) = chassis.get_mut(wheel.chassis) else {
            continue;
        };

        if !state.grounded {
            state.spin_speed_rad_per_sec *= 0.96;
            state.spin_angle_rad += state.spin_speed_rad_per_sec * dt;
            continue;
        }

        let up = state.contact_normal.normalize_or_zero();
        let chassis_up = transform.rotation * Vec3::Y;
        let chassis_forward = transform.rotation * Vec3::NEG_Z;
        let steer_rotation = Quat::from_axis_angle(chassis_up, state.steer_angle_rad);
        let wheel_forward = ((steer_rotation * chassis_forward)
            - up * (steer_rotation * chassis_forward).dot(up))
        .normalize_or_zero();
        let wheel_forward = if wheel_forward == Vec3::ZERO {
            (chassis_forward - up * chassis_forward.dot(up)).normalize_or_zero()
        } else {
            wheel_forward
        };
        let wheel_right = wheel_forward.cross(up).normalize_or_zero();
        let wheel_right = if wheel_right == Vec3::ZERO {
            transform.rotation * Vec3::X
        } else {
            wheel_right
        };

        let world_center_of_mass =
            transform.translation + transform.rotation * vehicle.center_of_mass_offset;
        let patch_velocity = forces.linear_velocity()
            + forces
                .angular_velocity()
                .cross(state.contact_point - world_center_of_mass);

        let longitudinal_speed_mps = patch_velocity.dot(wheel_forward);
        let lateral_speed_mps = patch_velocity.dot(wheel_right);
        let surface = surface_for_contact(state.contact_entity, &surfaces, &collider_of);
        let handbrake_amount = (resolved.handbrake * wheel.handbrake_factor).clamp(0.0, 1.0);
        let low_speed_boost = if vehicle.stability.low_speed_traction_speed_threshold_mps > 0.0
            && longitudinal_speed_mps.abs()
                < vehicle.stability.low_speed_traction_speed_threshold_mps
        {
            vehicle.stability.low_speed_traction_boost.max(1.0)
        } else {
            1.0
        };

        let longitudinal_limit = load_scaled_limit(
            state.load_newtons * wheel.tire.longitudinal_grip * surface.longitudinal_grip_scale,
            state.load_newtons,
            wheel.tire.nominal_load_newtons,
            wheel.tire.load_sensitivity,
        ) * low_speed_boost
            * (1.0 + (wheel.tire.handbrake_longitudinal_multiplier - 1.0) * handbrake_amount);
        let lateral_limit = load_scaled_limit(
            state.load_newtons * wheel.tire.lateral_grip * surface.lateral_grip_scale,
            state.load_newtons,
            wheel.tire.nominal_load_newtons,
            wheel.tire.load_sensitivity,
        ) * if longitudinal_speed_mps.abs()
            < vehicle.stability.low_speed_traction_speed_threshold_mps
        {
            wheel.tire.low_speed_lateral_multiplier.max(1.0)
        } else {
            1.0
        } * (1.0
            + (wheel.tire.handbrake_lateral_multiplier - 1.0) * handbrake_amount);

        let passive_longitudinal = -longitudinal_speed_mps
            * wheel.tire.longitudinal_stiffness
            * surface.rolling_drag_scale;
        let rolling_resistance = -longitudinal_speed_mps.signum()
            * wheel.tire.rolling_resistance_force_newtons
            * surface.rolling_drag_scale;
        let raw_longitudinal_force = wheel_internal.drive_force_request_newtons
            + wheel_internal.brake_force_request_newtons * surface.brake_scale
            + passive_longitudinal
            + rolling_resistance;
        let raw_lateral_force = -lateral_speed_mps.signum()
            * wheel.tire.lateral_stiffness
            * lateral_speed_mps
                .abs()
                .powf(wheel.tire.lateral_response_exponent.max(0.5));

        let scale = friction_circle_scale(
            raw_longitudinal_force,
            longitudinal_limit,
            raw_lateral_force,
            lateral_limit,
        );

        let longitudinal_force_newtons = raw_longitudinal_force * scale;
        let lateral_force_newtons = raw_lateral_force * scale;
        let total_force =
            wheel_forward * longitudinal_force_newtons + wheel_right * lateral_force_newtons;
        forces.apply_force_at_point(total_force, state.contact_point);

        state.longitudinal_speed_mps = longitudinal_speed_mps;
        state.lateral_speed_mps = lateral_speed_mps;
        state.longitudinal_force_newtons = longitudinal_force_newtons;
        state.lateral_force_newtons = lateral_force_newtons;
        state.spin_speed_rad_per_sec = longitudinal_speed_mps / wheel.radius_m.max(0.01);
        state.spin_angle_rad += state.spin_speed_rad_per_sec * dt;
    }
}
