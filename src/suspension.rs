use crate::{
    AxleAccumulator, GroundVehicle, GroundVehicleInternal, GroundVehicleWheel,
    GroundVehicleWheelInternal, GroundVehicleWheelState, SuspensionConfig, WheelSide,
};
use avian3d::prelude::*;
use bevy::prelude::*;

pub(crate) fn suspension_force(
    suspension: SuspensionConfig,
    current_length_m: f32,
    previous_length_m: f32,
    dt: f32,
) -> (f32, f32, f32) {
    let compression_m = (suspension.rest_length_m - current_length_m).max(0.0);
    let bump_stop_m = (suspension.min_length() - current_length_m).max(0.0);
    let suspension_velocity_mps = (previous_length_m - current_length_m) / dt.max(0.000_1);
    let force = compression_m * suspension.spring_strength_n_per_m
        + bump_stop_m * suspension.bump_stop_strength_n_per_m
        + suspension_velocity_mps * suspension.damper_strength_n_per_mps;

    (compression_m, suspension_velocity_mps, force.max(0.0))
}

pub(crate) fn reset_chassis_accumulators(mut chassis: Query<&mut GroundVehicleInternal>) {
    for mut internal in &mut chassis {
        internal.grounded_wheels = 0;
        internal.average_ground_normal_sum = Vec3::ZERO;
        internal.drive_factor_sum = 0.0;
        internal.drive_load_sum = 0.0;
        internal.left_drive_factor_sum = 0.0;
        internal.right_drive_factor_sum = 0.0;
        internal.left_drive_load_sum = 0.0;
        internal.right_drive_load_sum = 0.0;
        internal.axle_accumulators.fill(AxleAccumulator::default());
    }
}

pub(crate) fn sample_wheels_and_apply_suspension(
    time: Res<Time>,
    spatial_query: Option<SpatialQuery>,
    mut wheels: Query<(
        Entity,
        &GroundVehicleWheel,
        &mut GroundVehicleWheelState,
        &mut GroundVehicleWheelInternal,
    )>,
    mut chassis: Query<(
        Forces,
        &GroundVehicle,
        &Transform,
        &mut GroundVehicleInternal,
    )>,
) {
    let Some(spatial_query) = spatial_query else {
        return;
    };
    let dt = time.delta_secs().max(1.0 / 480.0);

    for (wheel_entity, wheel, mut state, mut wheel_internal) in &mut wheels {
        let Ok((mut forces, _vehicle, transform, mut chassis_internal)) =
            chassis.get_mut(wheel.chassis)
        else {
            continue;
        };

        let up = transform.rotation * Vec3::Y;
        let suspension_dir = Dir3::new(-up).unwrap_or(Dir3::NEG_Y);
        let origin = transform.transform_point(wheel.mount_point);
        let shape = Collider::sphere(wheel.radius_m);
        let mut filter = SpatialQueryFilter::default();
        filter.excluded_entities.insert(wheel_entity);
        filter.excluded_entities.insert(wheel.chassis);

        let max_length_m = wheel.suspension.max_length();
        let hit = spatial_query.cast_shape(
            &shape,
            origin,
            Quat::IDENTITY,
            suspension_dir,
            &ShapeCastConfig {
                max_distance: max_length_m,
                ignore_origin_penetration: false,
                ..default()
            },
            &filter,
        );

        let mut grounded = false;
        let mut contact_entity = None;
        let mut contact_point = origin + suspension_dir.as_vec3() * max_length_m;
        let mut contact_normal = up;
        let raw_length_m = if let Some(hit) = hit {
            if hit.normal1.dot(up) > 0.15 {
                grounded = true;
                contact_entity = Some(hit.entity);
                contact_point = hit.point1;
                contact_normal = hit.normal1.normalize_or_zero();
                hit.distance
            } else {
                max_length_m
            }
        } else {
            max_length_m
        };

        let clamped_length_m = raw_length_m.clamp(wheel.suspension.min_length(), max_length_m);
        let (compression_m, suspension_velocity_mps, suspension_force_newtons) = suspension_force(
            wheel.suspension,
            clamped_length_m,
            wheel_internal.previous_suspension_length_m.max(0.001),
            dt,
        );

        state.grounded = grounded;
        state.contact_entity = contact_entity;
        state.contact_point = contact_point;
        state.contact_normal = if grounded { contact_normal } else { up };
        state.suspension_length_m = clamped_length_m;
        state.suspension_compression_m = compression_m;
        state.suspension_velocity_mps = suspension_velocity_mps;
        state.suspension_force_newtons = if grounded {
            suspension_force_newtons
        } else {
            0.0
        };
        state.load_newtons = state.suspension_force_newtons;
        state.longitudinal_force_newtons = 0.0;
        state.lateral_force_newtons = 0.0;
        state.longitudinal_speed_mps = 0.0;
        state.lateral_speed_mps = 0.0;
        state.slip_ratio = 0.0;
        state.slip_angle_rad = 0.0;
        wheel_internal.previous_suspension_length_m = clamped_length_m;

        if wheel.drive_factor > 0.0 {
            chassis_internal.drive_factor_sum += wheel.drive_factor;
            match wheel.drive_side {
                WheelSide::Left => chassis_internal.left_drive_factor_sum += wheel.drive_factor,
                WheelSide::Right => chassis_internal.right_drive_factor_sum += wheel.drive_factor,
                WheelSide::Center => {}
            }
        }

        if grounded {
            let support_force = contact_normal * suspension_force_newtons;
            forces.apply_force_at_point(support_force, contact_point);
            chassis_internal.grounded_wheels = chassis_internal.grounded_wheels.saturating_add(1);
            chassis_internal.average_ground_normal_sum += contact_normal;

            if wheel.drive_factor > 0.0 {
                chassis_internal.drive_load_sum += state.load_newtons;
                match wheel.drive_side {
                    WheelSide::Left => chassis_internal.left_drive_load_sum += state.load_newtons,
                    WheelSide::Right => chassis_internal.right_drive_load_sum += state.load_newtons,
                    WheelSide::Center => {}
                }
            }

            let compression_ratio = ((max_length_m - clamped_length_m)
                / wheel.suspension.total_travel())
            .clamp(0.0, 1.0);
            let axle_slot =
                usize::from(wheel.axle).min(chassis_internal.axle_accumulators.len() - 1);
            let axle = &mut chassis_internal.axle_accumulators[axle_slot];
            match wheel.side {
                WheelSide::Left => {
                    axle.left_count = axle.left_count.saturating_add(1);
                    axle.left_compression_sum += compression_ratio;
                    axle.left_contact_sum += contact_point;
                }
                WheelSide::Right => {
                    axle.right_count = axle.right_count.saturating_add(1);
                    axle.right_compression_sum += compression_ratio;
                    axle.right_contact_sum += contact_point;
                }
                WheelSide::Center => {}
            }
        }
    }
}
