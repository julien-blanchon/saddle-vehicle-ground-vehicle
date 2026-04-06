use crate::{
    AxleDriveConfig, DifferentialConfig, DirectionChangeConfig, DirectionChangePolicy, DriveModel,
    FixedGearConfig, GearModel, GroundVehicle, GroundVehicleInternal, GroundVehicleResolvedIntent,
    GroundVehicleWheel, GroundVehicleWheelInternal, GroundVehicleWheelState, TrackDriveConfig,
    VehicleIntent, WheelSide, steering::speed_sensitive_factor,
};
use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;

pub(crate) fn resolve_direction_change_policy(
    drive: f32,
    brake: f32,
    forward_speed_mps: f32,
    direction_change: DirectionChangeConfig,
) -> (f32, f32) {
    let clamped_drive = drive.clamp(-1.0, 1.0);
    let clamped_brake = brake.clamp(0.0, 1.0);
    if direction_change.policy == DirectionChangePolicy::Immediate {
        return (clamped_drive, clamped_brake);
    }

    if forward_speed_mps.abs() > direction_change.speed_threshold_mps
        && clamped_drive.signum() != 0.0
        && clamped_drive.signum() != forward_speed_mps.signum()
    {
        (0.0, clamped_brake.max(clamped_drive.abs()))
    } else {
        (clamped_drive, clamped_brake)
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

fn direction_change_for(gear_model: GearModel) -> DirectionChangeConfig {
    match gear_model {
        GearModel::Automatic(config) => config.direction_change,
        GearModel::Fixed(config) => config.direction_change,
    }
}

fn coupling_speed_for(gear_model: GearModel) -> f32 {
    match gear_model {
        GearModel::Automatic(config) => config.coupling_speed_mps,
        GearModel::Fixed(config) => config.coupling_speed_mps,
    }
}

fn gear_ratio_for(gear_model: GearModel, gear: i8) -> f32 {
    match gear_model {
        GearModel::Automatic(config) => config.gear_ratio(gear),
        GearModel::Fixed(config) => config.gear_ratio(gear),
    }
}

fn select_automatic_gear(
    current_gear: i8,
    drive: f32,
    engine_rpm: f32,
    gearbox: crate::AutomaticGearboxConfig,
) -> i8 {
    if drive < -0.05 {
        return -1;
    }
    if drive.abs() < 0.01 {
        return current_gear.max(1);
    }

    let mut gear = current_gear.max(1);
    if engine_rpm >= gearbox.shift_up_rpm && gear < gearbox.max_forward_gear() {
        gear += 1;
    } else if engine_rpm <= gearbox.shift_down_rpm && gear > 1 {
        gear -= 1;
    }
    gear.clamp(1, gearbox.max_forward_gear())
}

fn select_fixed_gear(current_gear: i8, drive: f32, _gearbox: FixedGearConfig) -> i8 {
    if drive < -0.05 {
        -1
    } else if drive > 0.05 {
        1
    } else {
        current_gear.max(1)
    }
}

fn drive_model_settings(drive_model: DriveModel) -> (DifferentialConfig, f32, Option<f32>) {
    match drive_model {
        DriveModel::Axle(AxleDriveConfig {
            differential,
            drivetrain_efficiency,
        }) => (differential, drivetrain_efficiency, None),
        DriveModel::Track(TrackDriveConfig {
            differential,
            drivetrain_efficiency,
            turn_split,
        }) => (differential, drivetrain_efficiency, Some(turn_split)),
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

    if grounded_count > 0.0 {
        Some(grounded_rpm_sum / grounded_count)
    } else if all_count > 0.0 {
        Some(all_rpm_sum / all_count)
    } else {
        None
    }
}

pub(crate) fn resolve_vehicle_intent(
    mut chassis: Query<(
        &GroundVehicle,
        &VehicleIntent,
        &LinearVelocity,
        &Transform,
        &mut GroundVehicleResolvedIntent,
    )>,
) {
    for (vehicle, intent, linear_velocity, transform, mut resolved) in &mut chassis {
        let forward_speed_mps = linear_velocity.0.dot(*transform.forward());
        let direction_change = direction_change_for(vehicle.powertrain.gear_model);
        let (drive, brake) = resolve_direction_change_policy(
            intent.drive,
            intent.brake,
            forward_speed_mps,
            direction_change,
        );
        let turn = intent.turn.clamp(-1.0, 1.0);
        let auxiliary_brake = intent.auxiliary_brake.clamp(0.0, 1.0);
        let turn_speed_factor = speed_sensitive_factor(vehicle.steering, forward_speed_mps.abs());

        resolved.drive = drive;
        resolved.turn = turn;
        resolved.brake = brake;
        resolved.auxiliary_brake = auxiliary_brake;

        let (_, _, track_turn_split) = drive_model_settings(vehicle.powertrain.drive_model);
        if let Some(turn_split) = track_turn_split {
            let signed_turn = turn * turn_split * turn_speed_factor;
            resolved.left_drive = (drive - signed_turn).clamp(-1.0, 1.0);
            resolved.right_drive = (drive + signed_turn).clamp(-1.0, 1.0);
        } else {
            resolved.left_drive = drive;
            resolved.right_drive = drive;
        }
    }
}

pub(crate) fn update_powertrain_state(
    mut chassis: Query<(
        Entity,
        &GroundVehicle,
        &GroundVehicleResolvedIntent,
        &LinearVelocity,
        &Transform,
        &mut GroundVehicleInternal,
    )>,
    wheels: Query<(&GroundVehicleWheel, &GroundVehicleWheelState)>,
) {
    for (entity, vehicle, resolved, linear_velocity, transform, mut internal) in &mut chassis {
        let forward_speed_mps = linear_velocity.0.dot(*transform.forward());
        let (_, _, track_turn_split) = drive_model_settings(vehicle.powertrain.drive_model);
        let drive_side = if track_turn_split.is_some() {
            if resolved.left_drive.abs() > resolved.right_drive.abs() {
                Some(WheelSide::Left)
            } else if resolved.right_drive.abs() > resolved.left_drive.abs() {
                Some(WheelSide::Right)
            } else {
                None
            }
        } else {
            None
        };

        let wheel_rpm =
            drive_wheel_rpm_for_side(entity, drive_side, &wheels).unwrap_or_else(|| {
                let reference_radius = wheels
                    .iter()
                    .find(|(wheel, _)| wheel.chassis == entity && wheel.drive_factor > 0.0)
                    .map(|(wheel, _)| wheel.radius_m.max(0.05))
                    .unwrap_or(0.36);
                forward_speed_mps.abs() / reference_radius * 60.0 / std::f32::consts::TAU
            });

        let current_ratio = gear_ratio_for(vehicle.powertrain.gear_model, internal.selected_gear);
        let coupled_rpm = if current_ratio > 0.0 {
            wheel_rpm * current_ratio
        } else {
            vehicle.powertrain.engine.idle_rpm
        };
        let free_rev_target = vehicle.powertrain.engine.idle_rpm.lerp(
            vehicle.powertrain.engine.redline_rpm,
            resolved.drive.abs().clamp(0.0, 1.0),
        );
        let coupling = if internal.grounded_wheels > 0 {
            (forward_speed_mps.abs() / coupling_speed_for(vehicle.powertrain.gear_model).max(0.1))
                .clamp(0.5, 1.0)
        } else {
            0.15
        };
        let preview_rpm = free_rev_target.lerp(coupled_rpm, coupling).clamp(
            vehicle.powertrain.engine.idle_rpm,
            vehicle.powertrain.engine.redline_rpm,
        );

        internal.selected_gear = match vehicle.powertrain.gear_model {
            GearModel::Automatic(config) => {
                select_automatic_gear(internal.selected_gear, resolved.drive, preview_rpm, config)
            }
            GearModel::Fixed(config) => {
                select_fixed_gear(internal.selected_gear, resolved.drive, config)
            }
        };

        let selected_ratio = gear_ratio_for(vehicle.powertrain.gear_model, internal.selected_gear);
        let coupled_rpm = if selected_ratio > 0.0 {
            wheel_rpm * selected_ratio
        } else {
            vehicle.powertrain.engine.idle_rpm
        };
        internal.engine_rpm = free_rev_target.lerp(coupled_rpm, coupling).clamp(
            vehicle.powertrain.engine.idle_rpm,
            vehicle.powertrain.engine.redline_rpm,
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
        &GroundVehicleResolvedIntent,
        &GroundVehicleInternal,
    )>,
) {
    for (wheel, state, mut wheel_internal) in &mut wheels {
        let Ok((vehicle, resolved, vehicle_internal)) = chassis.get(wheel.chassis) else {
            continue;
        };

        let (differential, drivetrain_efficiency, track_turn_split) =
            drive_model_settings(vehicle.powertrain.drive_model);
        let (drive_factor_sum, drive_load_sum, requested_drive) = if track_turn_split.is_some() {
            match wheel.drive_side {
                WheelSide::Left => (
                    vehicle_internal.left_drive_factor_sum.max(0.001),
                    vehicle_internal.left_drive_load_sum,
                    resolved.left_drive,
                ),
                WheelSide::Right => (
                    vehicle_internal.right_drive_factor_sum.max(0.001),
                    vehicle_internal.right_drive_load_sum,
                    resolved.right_drive,
                ),
                WheelSide::Center => (
                    vehicle_internal.drive_factor_sum.max(0.001),
                    vehicle_internal.drive_load_sum,
                    resolved.drive,
                ),
            }
        } else {
            (
                vehicle_internal.drive_factor_sum.max(0.001),
                vehicle_internal.drive_load_sum,
                resolved.drive,
            )
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
        let split = differential_share(differential, factor_share, load_share);

        let gear_ratio = gear_ratio_for(
            vehicle.powertrain.gear_model,
            vehicle_internal.selected_gear,
        );
        let engine_torque_nm = vehicle
            .powertrain
            .engine
            .torque_at_rpm(vehicle_internal.engine_rpm)
            * requested_drive.abs();
        let signed_drive_force =
            if wheel.drive_factor > 0.0 && gear_ratio > 0.0 && requested_drive.abs() > 0.01 {
                let wheel_torque_nm = engine_torque_nm * gear_ratio * drivetrain_efficiency * split;
                wheel_torque_nm / wheel.radius_m.max(0.05) * requested_drive.signum()
            } else {
                0.0
            };

        let engine_brake_force =
            if resolved.drive.abs() < 0.05 && gear_ratio > 0.0 && wheel.drive_factor > 0.0 {
                vehicle.powertrain.engine.engine_brake_torque_nm
                    * gear_ratio
                    * drivetrain_efficiency
                    * split
                    / wheel.radius_m.max(0.05)
            } else {
                0.0
            };
        let brake_force_mag =
            vehicle.powertrain.brake_force_newtons * resolved.brake * wheel.brake_factor.max(0.0)
                + vehicle.powertrain.auxiliary_brake_force_newtons
                    * resolved.auxiliary_brake
                    * wheel.auxiliary_brake_factor.max(0.0)
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
