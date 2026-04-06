//! Skid-steer ground vehicle example — tracked/skid-steer vehicle with 3 axles.
//!
//! Demonstrates `DriveModel::Track`, a spool differential, and independent
//! track control.  All wheels are driven; steering works by varying left/right
//! track speeds.  WASD to steer/throttle, Space to brake, Shift for auxiliary brake,
//! R to reset.

use bevy::prelude::*;
use ground_vehicle_example_support as support;
use ground_vehicle::{
    AerodynamicsConfig, DifferentialConfig, DifferentialMode, DirectionChangeConfig,
    DirectionChangePolicy, DriveModel, EngineConfig, GearModel, GroundVehicle,
    GroundVehicleSurface, GroundVehicleWheel, GroundVehicleWheelVisual, PowertrainConfig,
    StabilityConfig, SteeringConfig, SteeringMode, SuspensionConfig, TireGripConfig,
    TrackDriveConfig, VehicleIntent, WheelSide,
};
use support::{
    ExampleDriver, ResetPose, ScriptedControlOverride, driver_actions, spawn_overlay,
    spawn_surface_box, spawn_world,
};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle skid_steer", true);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);
    spawn_surface_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Skid Pad",
        Vec3::new(36.0, 0.05, 36.0),
        Transform::from_xyz(0.0, 0.025, 0.0),
        Color::srgb(0.20, 0.23, 0.20),
        GroundVehicleSurface {
            rolling_drag_scale: 1.15,
            ..default()
        },
    );
    spawn_overlay(&mut commands, "ground_vehicle skid_steer");

    // ---------------------------------------------------------------------------
    // Skid-steer vehicle — 2 100 kg, 3 axles, all-wheel drive, spool diff
    // ---------------------------------------------------------------------------
    let vehicle = GroundVehicle {
        mass_kg: 2_100.0,
        angular_inertia_kgm2: Vec3::new(1_800.0, 2_600.0, 3_300.0),
        center_of_mass_offset: Vec3::new(0.0, -0.45, 0.0),
        steering: SteeringConfig {
            mode: SteeringMode::Disabled,                        // no wheel-angle steering
            max_angle_rad: 0.0,                                  // no wheel angle for skid steer
            ackermann_ratio: 0.0,
            minimum_speed_factor: 1.0,
            ..default()
        },
        powertrain: PowertrainConfig {
            engine: EngineConfig {
                peak_torque_nm: 720.0,
                peak_torque_rpm: 2_200.0,
                redline_rpm: 4_100.0,
                idle_torque_fraction: 0.52,
                redline_torque_fraction: 0.54,
                engine_brake_torque_nm: 180.0,
                ..default()
            },
            gear_model: GearModel::Fixed(ground_vehicle::FixedGearConfig {
                forward_ratio: 5.30 * 5.40,
                reverse_ratio: 5.10 * 5.40,
                coupling_speed_mps: 1.5,
                direction_change: DirectionChangeConfig {
                    policy: DirectionChangePolicy::Immediate,
                    ..default()
                },
                ..default()
            }),
            drive_model: DriveModel::Track(TrackDriveConfig {
                differential: DifferentialConfig {
                    mode: DifferentialMode::Spool,               // locked for skid steer
                    ..default()
                },
                turn_split: 0.92,
                ..default()
            }),
            brake_force_newtons: 15_000.0,
            auxiliary_brake_force_newtons: 6_000.0,
            ..default()
        },
        stability: StabilityConfig {
            anti_roll_force_n_per_ratio: 4_000.0,
            park_hold_force_newtons: 7_000.0,
            yaw_stability_torque_nm_per_radps: 1_100.0,
            ..default()
        },
        aerodynamics: AerodynamicsConfig {
            drag_force_per_speed_sq: 1.45,
            downforce_per_speed_sq: 0.04,
        },
    };

    let chassis_size = Vec3::new(2.30, 1.05, 4.90);
    let transform = Transform::from_xyz(0.0, 1.30, 12.0);

    // ---------------------------------------------------------------------------
    // Spawn chassis
    // ---------------------------------------------------------------------------
    let chassis_entity = commands
        .spawn((
            Name::new("Skid Vehicle"),
            ExampleDriver,
            vehicle,
            VehicleIntent::default(),
            ScriptedControlOverride::default(),
            avian3d::prelude::Mass(vehicle.mass_kg),
            avian3d::prelude::AngularInertia::new(vehicle.angular_inertia_kgm2),
            avian3d::prelude::CenterOfMass::new(
                vehicle.center_of_mass_offset.x,
                vehicle.center_of_mass_offset.y,
                vehicle.center_of_mass_offset.z,
            ),
            ResetPose {
                transform,
                linear_velocity: Vec3::ZERO,
                angular_velocity: Vec3::ZERO,
            },
            avian3d::prelude::Collider::cuboid(chassis_size.x, chassis_size.y, chassis_size.z),
            Mesh3d(meshes.add(Cuboid::new(chassis_size.x, chassis_size.y, chassis_size.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.28, 0.52, 0.35),
                perceptual_roughness: 0.58,
                metallic: 0.08,
                ..default()
            })),
            transform,
            driver_actions(),
            bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::ACTIVE,
        ))
        .id();

    // Roof — parented to chassis so it follows the vehicle
    commands.entity(chassis_entity).with_children(|parent| {
        parent.spawn((
            Name::new("Skid Vehicle Roof"),
            Mesh3d(meshes.add(Cuboid::new(
                chassis_size.x * 0.72,
                chassis_size.y * 0.42,
                chassis_size.z * 0.45,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.28, 0.52, 0.35).mix(&Color::WHITE, 0.18),
                perceptual_roughness: 0.46,
                ..default()
            })),
            Transform::from_xyz(0.0, chassis_size.y * 0.46, 0.12),
        ));
    });

    // ---------------------------------------------------------------------------
    // Suspension & tire — uniform across all 6 wheels
    // ---------------------------------------------------------------------------
    let suspension = SuspensionConfig {
        rest_length_m: 0.30,
        max_compression_m: 0.15,
        max_droop_m: 0.12,
        spring_strength_n_per_m: 36_000.0,
        damper_strength_n_per_mps: 3_800.0,
        bump_stop_strength_n_per_m: 20_000.0,
    };
    let tire = TireGripConfig {
        longitudinal_grip: 1.55,
        lateral_grip: 1.00,
        rolling_resistance_force_newtons: 60.0,
        ..default()
    };

    // ---------------------------------------------------------------------------
    // Wheels — 3 axles at z = -1.7, 0.0, 1.7; all driven, no steering angle
    // ---------------------------------------------------------------------------
    let z_positions = [-1.7_f32, 0.0, 1.7];
    let wheel_color = Color::srgb(0.07, 0.07, 0.08);
    let mut wheel_index = 0u32;

    for (axle, z) in z_positions.into_iter().enumerate() {
        for (side, x) in [(WheelSide::Left, -0.95_f32), (WheelSide::Right, 0.95_f32)] {
            let mount = Vec3::new(x, -0.28, z);
            let visual_entity = commands
                .spawn((
                    Name::new(format!("Skid Vehicle Wheel Visual {}", wheel_index + 1)),
                    Mesh3d(meshes.add(Cylinder::new(0.42, 0.28_f32.max(0.08)))),
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
                Name::new(format!("Skid Vehicle Wheel {}", wheel_index + 1)),
                GroundVehicleWheel {
                    chassis: chassis_entity,
                    axle: axle as u8,
                    side,
                    drive_side: side,
                    mount_point: mount,
                    radius_m: 0.42,
                    width_m: 0.28,
                    rotational_inertia_kgm2: 1.45,
                    steer_factor: 0.0,                           // no wheel angle
                    drive_factor: 1.0,                           // all driven
                    brake_factor: 1.0,
                    auxiliary_brake_factor: 0.6,
                    suspension,
                    tire,
                },
                GroundVehicleWheelVisual {
                    visual_entity,
                    base_rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                    ..default()
                },
            ));

            wheel_index += 1;
        }
    }
}
