use crate::{
    GroundVehicle, GroundVehicleResolvedIntent, GroundVehicleSurface, GroundVehicleWheel,
    GroundVehicleWheelInternal, GroundVehicleWheelState, TireModel,
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
    if let Ok(owner) = collider_of.get(entity)
        && let Ok(surface) = surfaces.get(owner.body)
    {
        return *surface;
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

pub(crate) fn magic_formula_response(x: f32, b: f32, c: f32, e: f32) -> f32 {
    let x = x.clamp(-4.0, 4.0);
    let bx = b.max(0.01) * x.abs();
    let atan_bx = bx.atan();
    let value = (c.max(0.1) * (bx - e.clamp(-1.0, 1.0) * (bx - atan_bx)).atan()).sin();
    value.copysign(x)
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
        &GroundVehicleResolvedIntent,
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

        let drive_torque_nm = wheel_internal.drive_force_request_newtons * wheel.radius_m;
        let brake_direction = if state.spin_speed_rad_per_sec.abs() > 0.1 {
            -state.spin_speed_rad_per_sec.signum()
        } else if state.longitudinal_speed_mps.abs() > 0.1 {
            -state.longitudinal_speed_mps.signum()
        } else {
            0.0
        };
        let brake_torque_nm =
            wheel_internal.brake_force_request_newtons.abs() * wheel.radius_m * brake_direction;
        let rolling_drag_torque_nm = -state.spin_speed_rad_per_sec.signum()
            * wheel.tire.rolling_resistance_force_newtons
            * wheel.radius_m;
        let rotational_inertia = wheel.rotational_inertia_kgm2.max(0.05);

        if !state.grounded {
            state.spin_speed_rad_per_sec +=
                (drive_torque_nm + brake_torque_nm + rolling_drag_torque_nm) / rotational_inertia
                    * dt;
            state.spin_speed_rad_per_sec *= 0.995;
            state.spin_angle_rad += state.spin_speed_rad_per_sec * dt;
            state.slip_ratio = 0.0;
            state.slip_angle_rad = 0.0;
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
        let auxiliary_brake_amount =
            (resolved.auxiliary_brake * wheel.auxiliary_brake_factor).clamp(0.0, 1.0);
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
            * (1.0
                + (wheel.tire.auxiliary_brake_longitudinal_multiplier - 1.0)
                    * auxiliary_brake_amount);
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
            + (wheel.tire.auxiliary_brake_lateral_multiplier - 1.0) * auxiliary_brake_amount);

        let predicted_spin_speed = state.spin_speed_rad_per_sec
            + (drive_torque_nm + brake_torque_nm + rolling_drag_torque_nm) / rotational_inertia
                * dt;
        let wheel_surface_speed_mps = predicted_spin_speed * wheel.radius_m;
        let slip_reference = longitudinal_speed_mps
            .abs()
            .max(wheel_surface_speed_mps.abs())
            .max(wheel.tire.low_speed_slip_reference_mps.max(0.5));
        let slip_ratio = (wheel_surface_speed_mps - longitudinal_speed_mps) / slip_reference;
        let slip_angle_rad =
            lateral_speed_mps.atan2(longitudinal_speed_mps.abs().max(slip_reference * 0.5));

        // Stiffness-based drag at low speed, clamped to a fraction of tire load
        // at higher speeds so it behaves like real rolling resistance and
        // doesn't limit the car's top speed.
        let passive_raw = -longitudinal_speed_mps
            * wheel.tire.longitudinal_stiffness
            * surface.rolling_drag_scale;
        let max_passive = state.load_newtons * 0.15;
        let passive_longitudinal = passive_raw.clamp(-max_passive, max_passive);
        let rolling_resistance = -longitudinal_speed_mps.signum()
            * wheel.tire.rolling_resistance_force_newtons
            * surface.rolling_drag_scale;

        let raw_longitudinal_force = match wheel.tire.model {
            TireModel::Linear => {
                wheel_internal.drive_force_request_newtons
                    + wheel_internal.brake_force_request_newtons * surface.brake_scale
                    + passive_longitudinal
                    + rolling_resistance
            }
            TireModel::MagicFormula => {
                let normalized_slip = slip_ratio
                    / wheel
                        .tire
                        .magic_formula
                        .longitudinal_peak_slip_ratio
                        .max(0.01);
                longitudinal_limit
                    * magic_formula_response(
                        normalized_slip,
                        wheel.tire.magic_formula.longitudinal_b,
                        wheel.tire.magic_formula.longitudinal_c,
                        wheel.tire.magic_formula.longitudinal_e,
                    )
                    + rolling_resistance
            }
        };
        let raw_lateral_force = match wheel.tire.model {
            TireModel::Linear => {
                -lateral_speed_mps.signum()
                    * wheel.tire.lateral_stiffness
                    * lateral_speed_mps
                        .abs()
                        .powf(wheel.tire.lateral_response_exponent.max(0.5))
            }
            TireModel::MagicFormula => {
                let normalized_angle = slip_angle_rad
                    / wheel
                        .tire
                        .magic_formula
                        .lateral_peak_slip_angle_rad
                        .max(1.0_f32.to_radians());
                -lateral_limit
                    * magic_formula_response(
                        normalized_angle,
                        wheel.tire.magic_formula.lateral_b,
                        wheel.tire.magic_formula.lateral_c,
                        wheel.tire.magic_formula.lateral_e,
                    )
            }
        };

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

        let patch_torque_nm = longitudinal_force_newtons * wheel.radius_m;
        state.spin_speed_rad_per_sec +=
            (drive_torque_nm + brake_torque_nm + rolling_drag_torque_nm - patch_torque_nm)
                / rotational_inertia
                * dt;
        // Pull wheel spin toward ground-implied speed.  This prevents
        // runaway wheelspin from the passive-longitudinal drag feedback
        // while preserving the car-level force balance.
        let ground_spin_rad_per_sec = longitudinal_speed_mps / wheel.radius_m.max(0.05);
        let spin_error = state.spin_speed_rad_per_sec - ground_spin_rad_per_sec;
        let correction = spin_error
            * (wheel.tire.longitudinal_stiffness * wheel.radius_m / rotational_inertia).min(60.0)
            * dt;
        state.spin_speed_rad_per_sec -= correction;
        if state.spin_speed_rad_per_sec.abs() < 0.01 && longitudinal_speed_mps.abs() < 0.1 {
            state.spin_speed_rad_per_sec = 0.0;
        }

        state.longitudinal_speed_mps = longitudinal_speed_mps;
        state.lateral_speed_mps = lateral_speed_mps;
        state.longitudinal_force_newtons = longitudinal_force_newtons;
        state.lateral_force_newtons = lateral_force_newtons;
        state.slip_ratio = slip_ratio;
        state.slip_angle_rad = slip_angle_rad;
        state.spin_angle_rad += state.spin_speed_rad_per_sec * dt;
    }
}
