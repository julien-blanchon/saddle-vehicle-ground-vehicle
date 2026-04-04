use avian3d::prelude::LinearVelocity;
use bevy::{ecs::system::RunSystemOnce, prelude::*};

use crate::{
    DifferentialConfig, DifferentialMode, DrivetrainConfig, EngineConfig, GroundVehicle,
    GroundVehicleInternal, GroundVehicleResolvedControl, GroundVehicleWheel,
    GroundVehicleWheelState, TransmissionConfig, WheelSide, drivetrain,
};

#[test]
fn engine_torque_curve_peaks_in_mid_range() {
    let engine = EngineConfig {
        idle_rpm: 900.0,
        peak_torque_nm: 460.0,
        peak_torque_rpm: 4_100.0,
        redline_rpm: 6_900.0,
        idle_torque_fraction: 0.40,
        redline_torque_fraction: 0.55,
        engine_brake_torque_nm: 100.0,
    };

    let idle = engine.torque_at_rpm(engine.idle_rpm);
    let peak = engine.torque_at_rpm(engine.peak_torque_rpm);
    let redline = engine.torque_at_rpm(engine.redline_rpm);

    assert!(peak > idle);
    assert!(peak > redline);
}

#[test]
fn update_drivetrain_state_upshifts_when_engine_rpm_is_high() {
    let mut app = App::new();
    let drivetrain = DrivetrainConfig {
        engine: EngineConfig {
            idle_rpm: 900.0,
            peak_torque_nm: 420.0,
            peak_torque_rpm: 4_200.0,
            redline_rpm: 6_800.0,
            ..default()
        },
        transmission: TransmissionConfig {
            automatic: true,
            forward_gears: [3.2, 2.1, 1.5, 1.1, 0.9, 0.8],
            forward_gear_count: 5,
            final_drive_ratio: 3.9,
            shift_up_rpm: 4_600.0,
            shift_down_rpm: 2_200.0,
            ..default()
        },
        differential: DifferentialConfig {
            mode: DifferentialMode::LimitedSlip,
            limited_slip_load_bias: 0.55,
        },
        ..default()
    };
    let chassis = app
        .world_mut()
        .spawn((
            GroundVehicle {
                drivetrain,
                ..default()
            },
            GroundVehicleResolvedControl {
                throttle: 0.95,
                ..default()
            },
            LinearVelocity::ZERO,
            Transform::default(),
            GroundVehicleInternal {
                grounded_wheels: 2,
                selected_gear: 1,
                ..default()
            },
        ))
        .id();

    app.world_mut().spawn((
        GroundVehicleWheel::default_rear(chassis, Vec3::new(-0.8, -0.2, 1.2), WheelSide::Left),
        GroundVehicleWheelState {
            spin_speed_rad_per_sec: 190.0,
            ..default()
        },
    ));
    app.world_mut().spawn((
        GroundVehicleWheel::default_rear(chassis, Vec3::new(0.8, -0.2, 1.2), WheelSide::Right),
        GroundVehicleWheelState {
            spin_speed_rad_per_sec: 190.0,
            ..default()
        },
    ));

    let _ = app
        .world_mut()
        .run_system_once(drivetrain::update_drivetrain_state);

    let internal = app
        .world_mut()
        .query::<&GroundVehicleInternal>()
        .single(app.world())
        .expect("vehicle internal state should exist");

    assert!(internal.selected_gear > 1);
    assert!(internal.engine_rpm >= drivetrain.engine.idle_rpm);
}
