use crate::{
    DriftStateChanged, GroundVehicle, GroundVehicleInternal, GroundVehicleReset,
    GroundVehicleResolvedControl, GroundVehicleTelemetry, GroundVehicleWheel,
    GroundVehicleWheelInternal, GroundVehicleWheelState, VehicleBecameAirborne, VehicleLanded,
    WheelGroundedChanged,
};
use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct GroundVehicleRuntime(pub bool);

pub(crate) fn activate_runtime(mut runtime: ResMut<GroundVehicleRuntime>) {
    runtime.0 = true;
}

pub(crate) fn deactivate_runtime(mut runtime: ResMut<GroundVehicleRuntime>) {
    runtime.0 = false;
}

pub(crate) fn runtime_is_active(runtime: Res<GroundVehicleRuntime>) -> bool {
    runtime.0
}

pub(crate) fn sync_ground_vehicle_properties(
    mut commands: Commands,
    vehicles: Query<(Entity, &GroundVehicle), Or<(Added<GroundVehicle>, Changed<GroundVehicle>)>>,
) {
    for (entity, vehicle) in &vehicles {
        commands.entity(entity).insert((
            Mass(vehicle.mass_kg),
            AngularInertia::new(vehicle.angular_inertia_kgm2),
            CenterOfMass::new(
                vehicle.center_of_mass_offset.x,
                vehicle.center_of_mass_offset.y,
                vehicle.center_of_mass_offset.z,
            ),
        ));
    }
}

pub(crate) fn sync_new_wheel_state(
    mut wheels: Query<
        (
            &GroundVehicleWheel,
            &mut GroundVehicleWheelState,
            &mut GroundVehicleWheelInternal,
        ),
        Added<GroundVehicleWheel>,
    >,
) {
    for (wheel, mut state, mut internal) in &mut wheels {
        let length = wheel.suspension.max_length();
        state.suspension_length_m = length;
        state.contact_normal = Vec3::Y;
        internal.previous_suspension_length_m = length;
    }
}

pub(crate) fn process_vehicle_resets(
    mut commands: Commands,
    mut chassis: Query<(Entity, &mut GroundVehicleInternal), With<GroundVehicleReset>>,
    mut wheels: Query<(
        &GroundVehicleWheel,
        &mut GroundVehicleWheelState,
        &mut GroundVehicleWheelInternal,
    )>,
) {
    for (entity, mut internal) in &mut chassis {
        *internal = GroundVehicleInternal::default();
        for (wheel, mut state, mut wheel_internal) in &mut wheels {
            if wheel.chassis == entity {
                let rest_length = wheel.suspension.max_length();
                *state = GroundVehicleWheelState {
                    suspension_length_m: rest_length,
                    contact_normal: Vec3::Y,
                    ..default()
                };
                *wheel_internal = GroundVehicleWheelInternal {
                    previous_suspension_length_m: rest_length,
                    ..default()
                };
            }
        }
        commands.entity(entity).remove::<GroundVehicleReset>();
    }
}

pub(crate) fn apply_stability_helpers(
    mut chassis: Query<(
        Entity,
        Forces,
        &GroundVehicle,
        &GroundVehicleResolvedControl,
        &Transform,
        &GroundVehicleInternal,
    )>,
) {
    for (_entity, mut forces, vehicle, control, transform, internal) in &mut chassis {
        let up = transform.rotation * Vec3::Y;
        let linear_velocity = forces.linear_velocity();
        let angular_velocity = forces.angular_velocity();

        for axle in internal.axle_accumulators {
            if axle.left_count == 0 || axle.right_count == 0 {
                continue;
            }
            let left_ratio = axle.left_compression_sum / f32::from(axle.left_count);
            let right_ratio = axle.right_compression_sum / f32::from(axle.right_count);
            let roll_delta = left_ratio - right_ratio;
            if roll_delta.abs() < 0.001 {
                continue;
            }

            let left_point = axle.left_contact_sum / f32::from(axle.left_count);
            let right_point = axle.right_contact_sum / f32::from(axle.right_count);
            let anti_roll_force = roll_delta * vehicle.stability.anti_roll_force_n_per_ratio;
            forces.apply_force_at_point(-up * anti_roll_force, left_point);
            forces.apply_force_at_point(up * anti_roll_force, right_point);
        }

        if internal.grounded_wheels > 0
            && control.brake > 0.05
            && linear_velocity.length() < vehicle.stability.park_hold_speed_threshold_mps
        {
            let average_ground_normal = (internal.average_ground_normal_sum
                / f32::from(internal.grounded_wheels))
            .normalize_or_zero();
            let gravity_force = Vec3::NEG_Y * vehicle.mass_kg * 9.81;
            let tangent_force =
                gravity_force - average_ground_normal * gravity_force.dot(average_ground_normal);
            let hold_force = tangent_force.clamp_length_max(
                vehicle.stability.park_hold_force_newtons * control.brake.max(0.25),
            );
            forces.apply_force(-hold_force);
        }

        let planar_speed = (linear_velocity - up * linear_velocity.dot(up)).length();
        if internal.grounded_wheels > 0
            && planar_speed > vehicle.stability.yaw_stability_speed_threshold_mps
        {
            let yaw_torque = Vec3::Y
                * (-angular_velocity.y * vehicle.stability.yaw_stability_torque_nm_per_radps);
            forces.apply_torque(yaw_torque);
        }

        if internal.grounded_wheels == 0
            && vehicle.stability.airborne_upright_torque_nm_per_rad > 0.0
        {
            let current_up = *transform.up();
            let alignment_axis = current_up.cross(Vec3::Y);
            if alignment_axis.length_squared() > 0.000_1 {
                let correction = alignment_axis.normalize()
                    * current_up.angle_between(Vec3::Y)
                    * vehicle.stability.airborne_upright_torque_nm_per_rad;
                let damping = Vec3::new(-angular_velocity.x, 0.0, -angular_velocity.z)
                    * (vehicle.stability.airborne_upright_torque_nm_per_rad * 0.15);
                forces.apply_torque(correction + damping);
            }
        }
    }
}

pub(crate) fn apply_aerodynamics(mut chassis: Query<(Forces, &GroundVehicle, &Transform)>) {
    for (mut forces, vehicle, transform) in &mut chassis {
        let linear_velocity = forces.linear_velocity();
        let speed = linear_velocity.length();
        if speed <= 0.01 {
            continue;
        }
        let drag = -linear_velocity * speed * vehicle.aerodynamics.drag_force_per_speed_sq;
        let downforce =
            -*transform.up() * speed * speed * vehicle.aerodynamics.downforce_per_speed_sq;
        forces.apply_force(drag + downforce);
    }
}

pub(crate) fn update_vehicle_telemetry(
    mut chassis: Query<(
        Entity,
        &Transform,
        &LinearVelocity,
        &GroundVehicle,
        &GroundVehicleInternal,
        &mut GroundVehicleTelemetry,
    )>,
    wheels: Query<(&GroundVehicleWheel, &GroundVehicleWheelState)>,
) {
    for (entity, transform, linear_velocity, vehicle, internal, mut telemetry) in &mut chassis {
        let speed_mps = linear_velocity.0.length();
        let forward_speed_mps = linear_velocity.0.dot(*transform.forward());
        let lateral_speed_mps = linear_velocity.0.dot(*transform.right());
        let airborne = internal.grounded_wheels == 0;

        let mut steer_sum = 0.0;
        let mut steer_count = 0_u32;
        let mut drift_sum = 0.0;
        let mut drift_count = 0_u32;

        for (wheel, state) in &wheels {
            if wheel.chassis != entity {
                continue;
            }
            steer_sum += state.steer_angle_rad.abs();
            steer_count += 1;

            if wheel.drive_factor > 0.0
                || wheel.handbrake_factor > 0.0
                || matches!(wheel.side, crate::WheelSide::Left | crate::WheelSide::Right)
            {
                drift_sum += (state.lateral_speed_mps.abs()
                    / (state.longitudinal_speed_mps.abs() + 2.0))
                    .clamp(0.0, 3.0);
                drift_count += 1;
            }
        }

        let drift_ratio = if drift_count > 0 {
            drift_sum / drift_count as f32
        } else {
            0.0
        };
        let drift_threshold = if internal.was_drifting {
            vehicle.stability.drift_exit_ratio
        } else {
            vehicle.stability.drift_entry_ratio
        };
        let drifting = !airborne && forward_speed_mps.abs() > 5.0 && drift_ratio >= drift_threshold;
        let average_ground_normal = if internal.grounded_wheels > 0 {
            (internal.average_ground_normal_sum / f32::from(internal.grounded_wheels))
                .normalize_or_zero()
        } else {
            Vec3::Y
        };

        telemetry.speed_mps = speed_mps;
        telemetry.forward_speed_mps = forward_speed_mps;
        telemetry.lateral_speed_mps = lateral_speed_mps;
        telemetry.grounded_wheels = internal.grounded_wheels;
        telemetry.airborne = airborne;
        telemetry.average_steer_angle_rad = if steer_count > 0 {
            steer_sum / steer_count as f32
        } else {
            0.0
        };
        telemetry.drift_ratio = drift_ratio;
        telemetry.drifting = drifting;
        telemetry.average_ground_normal = average_ground_normal;
        telemetry.engine_rpm = internal.engine_rpm;
        telemetry.selected_gear = internal.selected_gear;
    }
}

pub(crate) fn emit_wheel_grounded_messages(
    mut wheels: Query<(
        Entity,
        &GroundVehicleWheel,
        &GroundVehicleWheelState,
        &mut GroundVehicleWheelInternal,
    )>,
    writer: Option<MessageWriter<WheelGroundedChanged>>,
) {
    let Some(mut writer) = writer else {
        return;
    };
    for (wheel_entity, wheel, state, mut internal) in &mut wheels {
        if state.grounded != internal.previous_grounded {
            writer.write(WheelGroundedChanged {
                chassis: wheel.chassis,
                wheel: wheel_entity,
                grounded: state.grounded,
                surface_entity: state.contact_entity,
            });
        }
        internal.previous_grounded = state.grounded;
        internal.previous_contact_entity = state.contact_entity;
    }
}

pub(crate) fn emit_vehicle_state_messages(
    mut chassis: Query<(
        Entity,
        &LinearVelocity,
        &GroundVehicleTelemetry,
        &mut GroundVehicleInternal,
    )>,
    airborne_writer: Option<MessageWriter<VehicleBecameAirborne>>,
    landed_writer: Option<MessageWriter<VehicleLanded>>,
    drift_writer: Option<MessageWriter<DriftStateChanged>>,
) {
    let (Some(mut airborne_writer), Some(mut landed_writer), Some(mut drift_writer)) =
        (airborne_writer, landed_writer, drift_writer)
    else {
        return;
    };
    for (entity, linear_velocity, telemetry, mut internal) in &mut chassis {
        if telemetry.airborne && !internal.was_airborne {
            airborne_writer.write(VehicleBecameAirborne { chassis: entity });
        }
        if !telemetry.airborne && internal.was_airborne {
            landed_writer.write(VehicleLanded {
                chassis: entity,
                impact_speed_mps: linear_velocity.0.y.abs(),
                grounded_wheels: telemetry.grounded_wheels,
            });
        }
        if telemetry.drifting != internal.was_drifting {
            drift_writer.write(DriftStateChanged {
                chassis: entity,
                drifting: telemetry.drifting,
                drift_ratio: telemetry.drift_ratio,
            });
        }

        internal.was_airborne = telemetry.airborne;
        internal.was_drifting = telemetry.drifting;
    }
}
