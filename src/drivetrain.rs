use crate::{
    DifferentialMode, DrivetrainConfig, GroundVehicle, GroundVehicleControl, GroundVehicleInternal,
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
    drivetrain: DrivetrainConfig,
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
    differential: DifferentialMode,
    factor_share: f32,
    load_share: f32,
    limited_slip_load_bias: f32,
) -> f32 {
    match differential {
        DifferentialMode::Open => factor_share,
        DifferentialMode::Spool => load_share.max(0.0),
        DifferentialMode::LimitedSlip => {
            factor_share.lerp(load_share.max(0.0), limited_slip_load_bias.clamp(0.0, 1.0))
        }
    }
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
        let split = differential_share(
            vehicle.drivetrain.differential,
            factor_share,
            load_share,
            vehicle.drivetrain.limited_slip_load_bias,
        );

        let signed_drive_force = if requested_side_input >= 0.0 {
            vehicle.drivetrain.max_drive_force_newtons * requested_side_input.abs()
        } else {
            -vehicle.drivetrain.max_reverse_force_newtons * requested_side_input.abs()
        };

        let brake_force_mag =
            vehicle.drivetrain.brake_force_newtons * resolved.brake * wheel.brake_factor.max(0.0)
                + vehicle.drivetrain.handbrake_force_newtons
                    * resolved.handbrake
                    * wheel.handbrake_factor.max(0.0)
                + if resolved.throttle.abs() < 0.05 {
                    vehicle.drivetrain.engine_brake_force_newtons * wheel.drive_factor.max(0.0)
                } else {
                    0.0
                };

        let brake_sign = if state.longitudinal_speed_mps.abs() > 0.1 {
            -state.longitudinal_speed_mps.signum()
        } else if signed_drive_force.abs() > 0.1 {
            -signed_drive_force.signum()
        } else {
            0.0
        };

        wheel_internal.drive_force_request_newtons = signed_drive_force * split;
        wheel_internal.brake_force_request_newtons = brake_force_mag * brake_sign;
    }
}
