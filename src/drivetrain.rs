use crate::{
    DifferentialConfig, GroundVehicle, GroundVehicleControl, GroundVehicleInternal,
    GroundVehicleResolvedControl, GroundVehicleWheel, GroundVehicleWheelInternal,
    GroundVehicleWheelState, ReversePolicy, SteeringMode, WheelSide,
    steering::speed_sensitive_factor,
};
use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;

pub(crate) fn resolve_reverse_policy(
    throttle: f32,
    brake: f32,
    forward_speed_mps: f32,
    drivetrain: crate::DrivetrainConfig,
) -> (f32, f32) {
    let clamped_throttle = throttle.clamp(-1.0, 1.0);
    let clamped_brake = brake.clamp(0.0, 1.0);
    if drivetrain.reverse_policy == ReversePolicy::Immediate {
        return (clamped_throttle, clamped_brake);
    }

    if forward_speed_mps.abs() > drivetrain.reverse_speed_threshold_mps
        && clamped_throttle.signum() != 0.0
        && clamped_throttle.signum() != forward_speed_mps.signum()
    {
        (0.0, clamped_brake.max(clamped_throttle.abs()))
    } else {
        (clamped_throttle, clamped_brake)
    }
}

pub(crate) fn differential_share(
    differential: DifferentialConfig,
    factor_share: f32,
    load_share: f32,
) -> f32 {
    match differential.mode {
        crate::DifferentialMode::Open => factor_share,
        crate::DifferentialMode::Spool => load_share.max(0.0),
        crate::DifferentialMode::LimitedSlip => factor_share.lerp(
            load_share.max(0.0),
            differential.limited_slip_load_bias.clamp(0.0, 1.0),
        ),
    }
}

fn drive_wheel_rpm_for_side(
    chassis: Entity,
    drive_side: Option<WheelSide>,
    wheels: &Query<(&GroundVehicleWheel, &GroundVehicleWheelState)>,
) -> Option<f32> {
    let mut grounded_rpm_sum = 0.0;
    let mut grounded_count = 0.0;
    let mut all_rpm_sum = 0.0;
    let mut all_count = 0.0;

    for (wheel, state) in wheels.iter() {
        if wheel.chassis != chassis || wheel.drive_factor <= 0.0 {
            continue;
        }
        if let Some(side) = drive_side
            && wheel.drive_side != side
            && !matches!(wheel.drive_side, WheelSide::Center)
        {
            continue;
        }

        let rpm = state.spin_speed_rad_per_sec.abs() * 60.0 / std::f32::consts::TAU;
        all_rpm_sum += rpm;
        all_count += 1.0;
        if state.grounded {
            grounded_rpm_sum += rpm;
            grounded_count += 1.0;
        }
    }

    // Prefer grounded wheels — airborne wheels spin freely and inflate the
    // average, causing inappropriate upshifts.  Fall back to all drive
    // wheels when none are grounded (fully airborne).
    if grounded_count > 0.0 {
        Some(grounded_rpm_sum / grounded_count)
    } else if all_count > 0.0 {
        Some(all_rpm_sum / all_count)
    } else {
        None
    }
}

fn select_gear(
    current_gear: i8,
    throttle: f32,
    engine_rpm: f32,
    drivetrain: crate::DrivetrainConfig,
) -> i8 {
    if throttle < -0.05 {
        return -1;
    }
    if throttle.abs() < 0.01 {
        return current_gear.max(1);
    }

    let transmission = drivetrain.transmission;
    let mut gear = current_gear.max(1);
    if transmission.automatic {
        if engine_rpm >= transmission.shift_up_rpm && gear < transmission.max_forward_gear() {
            gear += 1;
        } else if engine_rpm <= transmission.shift_down_rpm && gear > 1 {
            gear -= 1;
        }
    }
    gear.clamp(1, transmission.max_forward_gear())
}

pub(crate) fn resolve_control_intent(
    mut chassis: Query<(
        &GroundVehicle,
        &GroundVehicleControl,
        &LinearVelocity,
        &Transform,
        &mut GroundVehicleResolvedControl,
    )>,
) {
    for (vehicle, control, linear_velocity, transform, mut resolved) in &mut chassis {
        let forward_speed_mps = linear_velocity.0.dot(*transform.forward());
        let (throttle, brake) = resolve_reverse_policy(
            control.throttle,
            control.brake,
            forward_speed_mps,
            vehicle.drivetrain,
        );
        let steering = control.steering.clamp(-1.0, 1.0);
        let handbrake = control.handbrake.clamp(0.0, 1.0);

        resolved.throttle = throttle;
        resolved.brake = brake;
        resolved.steering = steering;
        resolved.handbrake = handbrake;
        resolved.steer_speed_factor =
            speed_sensitive_factor(vehicle.steering, forward_speed_mps.abs());

        if vehicle.steering.mode == SteeringMode::SkidSteer {
            resolved.skid_left =
                (throttle - steering * vehicle.steering.skid_steer_turn_scale).clamp(-1.0, 1.0);
            resolved.skid_right =
                (throttle + steering * vehicle.steering.skid_steer_turn_scale).clamp(-1.0, 1.0);
        } else {
            resolved.skid_left = throttle;
            resolved.skid_right = throttle;
        }
    }
}

pub(crate) fn update_drivetrain_state(
    mut chassis: Query<(
        Entity,
        &GroundVehicle,
        &GroundVehicleResolvedControl,
        &LinearVelocity,
        &Transform,
        &mut GroundVehicleInternal,
    )>,
    wheels: Query<(&GroundVehicleWheel, &GroundVehicleWheelState)>,
) {
    for (entity, vehicle, resolved, linear_velocity, transform, mut internal) in &mut chassis {
        let forward_speed_mps = linear_velocity.0.dot(*transform.forward());
        let drive_side = if vehicle.steering.mode == SteeringMode::SkidSteer {
            if resolved.skid_left.abs() > resolved.skid_right.abs() {
                Some(WheelSide::Left)
            } else if resolved.skid_right.abs() > resolved.skid_left.abs() {
                Some(WheelSide::Right)
            } else {
                None
            }
        } else {
            None
        };

        let transmission = vehicle.drivetrain.transmission;
        let wheel_rpm =
            drive_wheel_rpm_for_side(entity, drive_side, &wheels).unwrap_or_else(|| {
                let reference_radius = wheels
                    .iter()
                    .find(|(wheel, _)| wheel.chassis == entity && wheel.drive_factor > 0.0)
                    .map(|(wheel, _)| wheel.radius_m.max(0.05))
                    .unwrap_or(0.36);
                forward_speed_mps.abs() / reference_radius * 60.0 / std::f32::consts::TAU
            });

        let current_ratio = transmission.gear_ratio(internal.selected_gear).max(0.0);
        let coupled_rpm = if current_ratio > 0.0 {
            wheel_rpm * current_ratio
        } else {
            vehicle.drivetrain.engine.idle_rpm
        };
        let free_rev_target = vehicle.drivetrain.engine.idle_rpm.lerp(
            vehicle.drivetrain.engine.redline_rpm,
            resolved.throttle.abs().clamp(0.0, 1.0),
        );
        let coupling = if internal.grounded_wheels > 0 {
            (forward_speed_mps.abs() / transmission.clutch_coupling_speed_mps.max(0.1))
                .clamp(0.5, 1.0)
        } else {
            0.15
        };
        let preview_rpm = free_rev_target.lerp(coupled_rpm, coupling).clamp(
            vehicle.drivetrain.engine.idle_rpm,
            vehicle.drivetrain.engine.redline_rpm,
        );

        internal.selected_gear = select_gear(
            internal.selected_gear,
            resolved.throttle,
            preview_rpm,
            vehicle.drivetrain,
        );

        let selected_ratio = transmission.gear_ratio(internal.selected_gear).max(0.0);
        let coupled_rpm = if selected_ratio > 0.0 {
            wheel_rpm * selected_ratio
        } else {
            vehicle.drivetrain.engine.idle_rpm
        };
        internal.engine_rpm = free_rev_target.lerp(coupled_rpm, coupling).clamp(
            vehicle.drivetrain.engine.idle_rpm,
            vehicle.drivetrain.engine.redline_rpm,
        );
    }
}

pub(crate) fn resolve_wheel_force_requests(
    mut wheels: Query<(
        &GroundVehicleWheel,
        &GroundVehicleWheelState,
        &mut GroundVehicleWheelInternal,
    )>,
    chassis: Query<(
        &GroundVehicle,
        &GroundVehicleResolvedControl,
        &GroundVehicleInternal,
    )>,
) {
    for (wheel, state, mut wheel_internal) in &mut wheels {
        let Ok((vehicle, resolved, vehicle_internal)) = chassis.get(wheel.chassis) else {
            continue;
        };

        let (drive_factor_sum, drive_load_sum, requested_side_input) = match vehicle.steering.mode {
            SteeringMode::SkidSteer => match wheel.drive_side {
                WheelSide::Left => (
                    vehicle_internal.left_drive_factor_sum.max(0.001),
                    vehicle_internal.left_drive_load_sum,
                    resolved.skid_left,
                ),
                WheelSide::Right => (
                    vehicle_internal.right_drive_factor_sum.max(0.001),
                    vehicle_internal.right_drive_load_sum,
                    resolved.skid_right,
                ),
                WheelSide::Center => (
                    vehicle_internal.drive_factor_sum.max(0.001),
                    vehicle_internal.drive_load_sum,
                    resolved.throttle,
                ),
            },
            SteeringMode::Road => (
                vehicle_internal.drive_factor_sum.max(0.001),
                vehicle_internal.drive_load_sum,
                resolved.throttle,
            ),
        };

        let factor_share = if wheel.drive_factor > 0.0 {
            wheel.drive_factor / drive_factor_sum
        } else {
            0.0
        };
        let load_share = if drive_load_sum > 0.0 {
            state.load_newtons / drive_load_sum
        } else {
            factor_share
        };
        let split = differential_share(vehicle.drivetrain.differential, factor_share, load_share);

        let gear_ratio = vehicle
            .drivetrain
            .transmission
            .gear_ratio(vehicle_internal.selected_gear)
            .max(0.0);
        let engine_torque_nm = vehicle
            .drivetrain
            .engine
            .torque_at_rpm(vehicle_internal.engine_rpm)
            * resolved.throttle.abs();
        let signed_drive_force = if wheel.drive_factor > 0.0
            && gear_ratio > 0.0
            && requested_side_input.abs() > 0.01
        {
            let wheel_torque_nm =
                engine_torque_nm * gear_ratio * vehicle.drivetrain.drivetrain_efficiency * split;
            wheel_torque_nm / wheel.radius_m.max(0.05) * requested_side_input.signum()
        } else {
            0.0
        };

        let engine_brake_force = if resolved.throttle.abs() < 0.05 && gear_ratio > 0.0 {
            vehicle.drivetrain.engine.engine_brake_torque_nm
                * gear_ratio
                * vehicle.drivetrain.drivetrain_efficiency
                * wheel.drive_factor.max(0.0)
                / wheel.radius_m.max(0.05)
        } else {
            0.0
        };
        let brake_force_mag =
            vehicle.drivetrain.brake_force_newtons * resolved.brake * wheel.brake_factor.max(0.0)
                + vehicle.drivetrain.handbrake_force_newtons
                    * resolved.handbrake
                    * wheel.handbrake_factor.max(0.0)
                + engine_brake_force;

        let brake_sign = if state.longitudinal_speed_mps.abs() > 0.1 {
            -state.longitudinal_speed_mps.signum()
        } else if signed_drive_force.abs() > 0.1 {
            -signed_drive_force.signum()
        } else {
            0.0
        };

        wheel_internal.drive_force_request_newtons = signed_drive_force;
        wheel_internal.brake_force_request_newtons = brake_force_mag * brake_sign;
    }
}
