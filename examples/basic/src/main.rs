//! Basic ground vehicle example — simplest 2-axle front-steer car.
//!
//! Shows every vehicle component with all fields visible so readers can see
//! exactly what goes into a minimal drivable car. WASD to steer/throttle,
//! Space to brake, Shift for auxiliary brake, R to reset.

use bevy::prelude::*;
use ground_vehicle_example_support as support;
use ground_vehicle::{
    AutomaticGearboxConfig, AxleDriveConfig, DifferentialConfig, DifferentialMode, DriveModel,
    EngineConfig, GearModel, GroundVehicle, GroundVehicleWheel, GroundVehicleWheelVisual,
    PowertrainConfig, SteeringConfig, SuspensionConfig, TireGripConfig, VehicleIntent, WheelSide,
};
use support::{
    ExampleDriver, ResetPose, ScriptedControlOverride, driver_actions, spawn_overlay, spawn_world,
};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle basic", false);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);
    spawn_overlay(&mut commands, "ground_vehicle basic");

    // ---------------------------------------------------------------------------
    // Vehicle configuration — a simple compact hatchback
    // ---------------------------------------------------------------------------
    let vehicle = GroundVehicle {
        mass_kg: 1_350.0,                                       // default
        angular_inertia_kgm2: Vec3::new(900.0, 1_100.0, 1_400.0),
        center_of_mass_offset: Vec3::new(0.0, -0.38, 0.0),
        steering: SteeringConfig {
            max_angle_rad: 29.0_f32.to_radians(),               // 29 deg lock
            ..default()
        },
        powertrain: PowertrainConfig {
            engine: EngineConfig {
                peak_torque_nm: 380.0,
                peak_torque_rpm: 4_200.0,
                redline_rpm: 6_500.0,
                engine_brake_torque_nm: 90.0,
                ..default()
            },
            gear_model: GearModel::Automatic(AutomaticGearboxConfig {
                final_drive_ratio: 3.70,
                forward_gears: [3.60, 2.19, 1.46, 1.09, 0.87, 0.72],
                forward_gear_count: 5,
                reverse_ratio: 3.18,
                shift_up_rpm: 5_700.0,
                shift_down_rpm: 2_450.0,
                ..default()
            }),
            drive_model: DriveModel::Axle(AxleDriveConfig {
                differential: DifferentialConfig {
                    mode: DifferentialMode::Open,
                    ..default()
                },
                ..default()
            }),
            brake_force_newtons: 15_000.0,
            ..default()
        },
        ..default()
    };

    let chassis_size = Vec3::new(1.85, 0.72, 4.20);
    let transform = Transform::from_xyz(0.0, 1.25, 18.0);

    // ---------------------------------------------------------------------------
    // Spawn chassis entity
    // ---------------------------------------------------------------------------
    let chassis_entity = commands
        .spawn((
            Name::new("Basic Hatchback"),
            ExampleDriver,
            vehicle,
            VehicleIntent::default(),
            ScriptedControlOverride::default(),
            avian3d::prelude::Mass(vehicle.mass_kg),
            avian3d::prelude::AngularInertia::new(vehicle.angular_inertia_kgm2),
            avian3d::prelude::CenterOfMass::new(0.0, -0.38, 0.0),
            ResetPose {
                transform,
                linear_velocity: Vec3::ZERO,
                angular_velocity: Vec3::ZERO,
            },
            avian3d::prelude::Collider::cuboid(chassis_size.x, chassis_size.y, chassis_size.z),
            Mesh3d(meshes.add(Cuboid::new(chassis_size.x, chassis_size.y, chassis_size.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.82, 0.21, 0.19),
                perceptual_roughness: 0.58,
                metallic: 0.08,
                ..default()
            })),
            transform,
            driver_actions(),
            bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::ACTIVE,
        ))
        .id();

    // Decorative roof — parented to chassis so it follows the vehicle
    commands.entity(chassis_entity).with_children(|parent| {
        parent.spawn((
            Name::new("Basic Hatchback Roof"),
            Mesh3d(meshes.add(Cuboid::new(
                chassis_size.x * 0.72,
                chassis_size.y * 0.42,
                chassis_size.z * 0.45,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.82, 0.21, 0.19).mix(&Color::WHITE, 0.18),
                perceptual_roughness: 0.46,
                ..default()
            })),
            Transform::from_xyz(0.0, chassis_size.y * 0.46, 0.12),
        ));
    });

    // ---------------------------------------------------------------------------
    // Suspension & tire configs per axle
    // ---------------------------------------------------------------------------
    let front_suspension = SuspensionConfig {
        rest_length_m: 0.34,
        max_compression_m: 0.16,
        max_droop_m: 0.15,
        spring_strength_n_per_m: 28_000.0,
        damper_strength_n_per_mps: 3_500.0,
        bump_stop_strength_n_per_m: 18_000.0,
    };
    let rear_suspension = SuspensionConfig {
        spring_strength_n_per_m: 30_000.0,
        ..front_suspension
    };
    let front_tire = TireGripConfig {
        longitudinal_grip: 1.55,
        lateral_grip: 1.20,
        ..default()
    };
    let rear_tire = TireGripConfig {
        longitudinal_grip: 1.45,
        lateral_grip: 1.10,
        ..default()
    };

    // ---------------------------------------------------------------------------
    // Wheels — front axle (axle 0): steered + driven
    // ---------------------------------------------------------------------------
    let wheel_specs: [(u8, WheelSide, Vec3, f32, f32, f32, f32, f32, f32, f32, SuspensionConfig, TireGripConfig); 4] = [
        //                                                                     steer drive brake handbrake
        // Front-left (steered + driven)
        (0, WheelSide::Left,  Vec3::new(-0.82, -0.20, -1.24), 0.36, 0.24, 1.02, 1.0, 1.0, 1.0, 0.0, front_suspension, front_tire),
        // Front-right (steered + driven)
        (0, WheelSide::Right, Vec3::new( 0.82, -0.20, -1.24), 0.36, 0.24, 1.02, 1.0, 1.0, 1.0, 0.0, front_suspension, front_tire),
        // Rear-left (driven + handbrake)
        (1, WheelSide::Left,  Vec3::new(-0.82, -0.20,  1.20), 0.37, 0.26, 1.10, 0.0, 1.0, 1.0, 1.0, rear_suspension, rear_tire),
        // Rear-right (driven + handbrake)
        (1, WheelSide::Right, Vec3::new( 0.82, -0.20,  1.20), 0.37, 0.26, 1.10, 0.0, 1.0, 1.0, 1.0, rear_suspension, rear_tire),
    ];

    let wheel_color = Color::srgb(0.12, 0.12, 0.13);
    for (i, (axle, side, mount, radius, width, inertia, steer, drive, brake, handbrake, susp, tire)) in wheel_specs.into_iter().enumerate() {
        let visual_entity = commands
            .spawn((
                Name::new(format!("Basic Hatchback Wheel Visual {}", i + 1)),
                Mesh3d(meshes.add(Cylinder::new(radius, width.max(0.08)))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: wheel_color,
                    perceptual_roughness: 0.92,
                    metallic: 0.02,
                    ..default()
                })),
                Transform::from_translation(transform.transform_point(mount)),
            ))
            .id();

        commands.spawn((
            Name::new(format!("Basic Hatchback Wheel {}", i + 1)),
            GroundVehicleWheel {
                chassis: chassis_entity,
                axle,
                side,
                drive_side: side,
                mount_point: mount,
                radius_m: radius,
                width_m: width,
                rotational_inertia_kgm2: inertia,
                steer_factor: steer,
                drive_factor: drive,
                brake_factor: brake,
                auxiliary_brake_factor: handbrake,
                suspension: susp,
                tire,
            },
            GroundVehicleWheelVisual {
                visual_entity,
                base_rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                ..default()
            },
        ));
    }
}
