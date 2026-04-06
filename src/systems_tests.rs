use std::time::Duration;

use avian3d::prelude::ColliderOf;
use bevy::{
    ecs::schedule::ScheduleLabel,
    ecs::system::{RunSystemOnce, SystemState},
    prelude::*,
    time::TimeUpdateStrategy,
    transform::TransformPlugin,
};

use crate::{
    DifferentialConfig, DifferentialMode, DirectionChangeConfig, DirectionChangePolicy, DriveModel,
    GroundVehicle, GroundVehiclePlugin, GroundVehicleSurface, GroundVehicleTelemetry,
    GroundVehicleWheel, GroundVehicleWheelState, GroundVehicleWheelVisual, PowertrainConfig,
    SteeringConfig, TrackDriveConfig, VehicleBecameAirborne, VehicleIntent, VehicleLanded,
    WheelGroundedChanged, WheelSide, drivetrain, grip, steering, suspension,
};

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivate;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, TransformPlugin));
    app.insert_resource(Time::<Fixed>::from_hz(60.0));
    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
        1.0 / 60.0,
    )));
    app.init_schedule(NeverDeactivate);
    app.add_plugins(GroundVehiclePlugin::new(
        Startup,
        NeverDeactivate,
        FixedUpdate,
    ));
    app
}

fn spawn_test_vehicle(app: &mut App) -> (Entity, Entity, Entity) {
    let vehicle = GroundVehicle::default();
    let chassis = app
        .world_mut()
        .spawn((
            Name::new("Test Chassis"),
            vehicle,
            Transform::from_xyz(0.0, 1.15, 0.0),
        ))
        .id();
    let visual = app
        .world_mut()
        .spawn((
            Name::new("Test Wheel Visual"),
            Transform::default(),
            GlobalTransform::default(),
        ))
        .id();
    let wheel = app
        .world_mut()
        .spawn((
            Name::new("Front Left Wheel"),
            GroundVehicleWheel::default_front(
                chassis,
                Vec3::new(-0.82, -0.15, -1.25),
                WheelSide::Left,
            ),
            GroundVehicleWheelVisual {
                visual_entity: visual,
                base_rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                ..default()
            },
        ))
        .id();

    (chassis, wheel, visual)
}

#[test]
fn plugin_builds_with_minimal_plugins_and_no_gizmo_storage() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_schedule(NeverDeactivate);
    app.add_plugins(GroundVehiclePlugin::new(Startup, NeverDeactivate, Update));
    app.update();
}

#[test]
fn messages_register_correctly() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_schedule(NeverDeactivate);
    app.add_plugins(GroundVehiclePlugin::new(Startup, NeverDeactivate, Update));

    assert!(
        app.world()
            .contains_resource::<Messages<WheelGroundedChanged>>()
    );
    assert!(
        app.world()
            .contains_resource::<Messages<VehicleBecameAirborne>>()
    );
    assert!(app.world().contains_resource::<Messages<VehicleLanded>>());
}

#[test]
fn suspension_force_tracks_compression_and_damping() {
    let suspension = crate::SuspensionConfig {
        rest_length_m: 0.4,
        max_compression_m: 0.2,
        max_droop_m: 0.2,
        spring_strength_n_per_m: 10_000.0,
        damper_strength_n_per_mps: 2_000.0,
        bump_stop_strength_n_per_m: 5_000.0,
    };
    let (compression_m, velocity_mps, force) =
        suspension::suspension_force(suspension, 0.28, 0.34, 1.0 / 60.0);

    assert!(compression_m > 0.11 && compression_m < 0.13);
    assert!(velocity_mps > 3.0);
    assert!(force > 0.0);
}

#[test]
fn ackermann_inside_wheel_turns_more() {
    let (left, right) = steering::ackermann_pair(0.4, 2.7, 1.6, 1.0);
    assert!(left > right);
    let (left_opposite, right_opposite) = steering::ackermann_pair(-0.4, 2.7, 1.6, 1.0);
    assert!(left_opposite.abs() < right_opposite.abs());
}

#[test]
fn ackermann_geometry_derives_from_wheel_layout() {
    let chassis = Entity::from_bits(1);
    let wheels = [
        GroundVehicleWheel::default_front(chassis, Vec3::new(-0.82, -0.15, -1.25), WheelSide::Left),
        GroundVehicleWheel::default_front(chassis, Vec3::new(0.82, -0.15, -1.25), WheelSide::Right),
        GroundVehicleWheel::default_rear(chassis, Vec3::new(-0.82, -0.15, 1.20), WheelSide::Left),
        GroundVehicleWheel::default_rear(chassis, Vec3::new(0.82, -0.15, 1.20), WheelSide::Right),
    ];

    let (wheelbase_m, track_width_m) =
        steering::derive_ackermann_geometry(wheels.iter()).expect("geometry should derive");

    assert!((wheelbase_m - 2.45).abs() < 0.01);
    assert!((track_width_m - 1.64).abs() < 0.01);
}

#[test]
fn direction_change_policy_brakes_before_reversing_when_requested() {
    let direction_change = DirectionChangeConfig {
        policy: DirectionChangePolicy::StopThenChange,
        speed_threshold_mps: 1.0,
    };
    let (drive, brake) =
        drivetrain::resolve_direction_change_policy(-1.0, 0.0, 4.0, direction_change);
    assert_eq!(drive, 0.0);
    assert!(brake > 0.99);
}

#[test]
fn limited_slip_prefers_loaded_side_more_than_open_diff() {
    let open = drivetrain::differential_share(
        DifferentialConfig {
            mode: DifferentialMode::Open,
            limited_slip_load_bias: 0.55,
        },
        0.5,
        0.8,
    );
    let limited = drivetrain::differential_share(
        DifferentialConfig {
            mode: DifferentialMode::LimitedSlip,
            limited_slip_load_bias: 0.55,
        },
        0.5,
        0.8,
    );
    let spool = drivetrain::differential_share(
        DifferentialConfig {
            mode: DifferentialMode::Spool,
            limited_slip_load_bias: 0.55,
        },
        0.5,
        0.8,
    );
    assert!((open - 0.5).abs() < f32::EPSILON);
    assert!(limited > open && limited < spool);
    assert!((spool - 0.8).abs() < f32::EPSILON);
}

#[test]
fn track_drive_splits_left_and_right_drive() {
    let mut app = App::new();
    app.world_mut().spawn((
        GroundVehicle {
            steering: SteeringConfig {
                mode: crate::SteeringMode::Disabled,
                ..default()
            },
            powertrain: PowertrainConfig {
                drive_model: DriveModel::Track(TrackDriveConfig {
                    turn_split: 0.8,
                    ..default()
                }),
                ..default()
            },
            ..default()
        },
        VehicleIntent {
            drive: 0.5,
            turn: 0.75,
            ..default()
        },
        crate::GroundVehicleResolvedIntent::default(),
        avian3d::prelude::LinearVelocity::ZERO,
        Transform::default(),
    ));

    let _ = app
        .world_mut()
        .run_system_once(drivetrain::resolve_vehicle_intent);

    let resolved = {
        let world = app.world_mut();
        let mut query = world.query::<&crate::GroundVehicleResolvedIntent>();
        query
            .single(world)
            .expect("resolved intent should exist for the test vehicle")
            .to_owned()
    };
    assert!(resolved.left_drive < resolved.right_drive);
    assert!((resolved.left_drive + 0.1).abs() < 0.001);
    assert!((resolved.right_drive - 1.0).abs() < 0.001);
}

#[test]
fn wheel_state_and_visuals_update_after_fixed_ticks() {
    let mut app = test_app();
    let (chassis, wheel, visual) = spawn_test_vehicle(&mut app);

    for _ in 0..2 {
        app.update();
    }

    let suspension_length_m = {
        let world = app.world_mut();
        let mut query = world.query::<&GroundVehicleWheelState>();
        query
            .get(world, wheel)
            .expect("wheel runtime state should exist")
            .suspension_length_m
    };
    let grounded_wheels = {
        let world = app.world_mut();
        let mut query = world.query::<&GroundVehicleTelemetry>();
        query
            .get(world, chassis)
            .expect("telemetry should exist")
            .grounded_wheels
    };
    let wheel_visual = {
        let world = app.world_mut();
        let mut query = world.query::<&Transform>();
        query
            .get(world, visual)
            .expect("wheel visual transform should exist")
            .translation
    };

    let chassis_translation = app
        .world()
        .get::<Transform>(chassis)
        .expect("chassis transform should exist")
        .translation;
    let expected_visual =
        chassis_translation + Vec3::new(-0.82, -0.15 - suspension_length_m, -1.25);

    assert!((suspension_length_m - 0.54).abs() < 0.001);
    assert_eq!(grounded_wheels, 0);
    assert!(wheel_visual.distance(expected_visual) < 0.001);
}

#[test]
fn surface_lookup_uses_collider_owner_surface() {
    let mut world = World::new();
    let owner = world
        .spawn(GroundVehicleSurface {
            lateral_grip_scale: 0.6,
            ..default()
        })
        .id();
    let collider = world.spawn(ColliderOf { body: owner }).id();

    let mut system_state: SystemState<(Query<&GroundVehicleSurface>, Query<&ColliderOf>)> =
        SystemState::new(&mut world);
    let (surfaces, collider_of) = system_state.get(&world);
    let surface = grip::surface_for_contact(Some(collider), &surfaces, &collider_of);

    assert!((surface.lateral_grip_scale - 0.6).abs() < f32::EPSILON);
}

#[test]
fn grounded_messages_emit_when_wheel_contacts_surface() {
    let mut app = test_app();
    let (chassis, wheel, _visual) = spawn_test_vehicle(&mut app);

    {
        let mut state = app
            .world_mut()
            .get_mut::<GroundVehicleWheelState>(wheel)
            .expect("wheel state should exist");
        state.grounded = true;
        state.contact_entity = Some(chassis);
    }

    let mut saw_grounded_message = false;
    for _ in 0..2 {
        app.update();
        let messages = app
            .world_mut()
            .resource_mut::<Messages<WheelGroundedChanged>>()
            .drain()
            .collect::<Vec<_>>();
        if messages.iter().any(|message| message.grounded) {
            saw_grounded_message = true;
            break;
        }
    }

    assert!(saw_grounded_message);
}

#[test]
fn steering_speed_factor_reduces_at_speed() {
    let steering = SteeringConfig::default();
    let low = steering::speed_sensitive_factor(steering, 0.0);
    let high = steering::speed_sensitive_factor(steering, 40.0);
    assert!(low > high);
    assert!((low - 1.0).abs() < f32::EPSILON);
}
