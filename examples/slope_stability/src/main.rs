//! Slope stability ground vehicle example — off-road rover on inclines.
//!
//! Demonstrates suspension articulation, per-wheel grip/load tuning, limited-slip
//! differential, and stability parameters for a vehicle designed to handle steep
//! grades and off-camber terrain. WASD to steer/throttle, Space to brake,
//! Shift for auxiliary brake, R to reset.

use bevy::prelude::*;
use ground_vehicle::{
    AerodynamicsConfig, AutomaticGearboxConfig, AxleDriveConfig, DifferentialConfig,
    DifferentialMode, DriveModel, EngineConfig, GearModel, GroundVehicle, GroundVehicleWheel,
    GroundVehicleWheelVisual, PowertrainConfig, StabilityConfig, SteeringConfig, SuspensionConfig,
    TireGripConfig, VehicleIntent, WheelSide,
};
use ground_vehicle_example_support as support;
use support::{
    ExampleDriver, ResetPose, ScriptedControlOverride, driver_actions, spawn_overlay, spawn_ramp,
    spawn_world,
};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle slope_stability", true);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);

    // Ramps to test articulation
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Main Ramp",
        Vec3::new(10.0, 0.8, 28.0),
        Transform::from_xyz(0.0, 1.4, 0.0).with_rotation(Quat::from_rotation_x(-0.26)),
        Color::srgb(0.46, 0.38, 0.23),
        default(),
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Off Camber Pad",
        Vec3::new(8.0, 0.6, 14.0),
        Transform::from_xyz(-10.0, 0.9, -16.0)
            .with_rotation(Quat::from_rotation_x(-0.18) * Quat::from_rotation_z(0.14)),
        Color::srgb(0.36, 0.34, 0.19),
        default(),
    );
    spawn_overlay(&mut commands, "ground_vehicle slope_stability");

    // ---------------------------------------------------------------------------
    // Off-road rover — lightweight 4WD with long-travel suspension
    // ---------------------------------------------------------------------------
    let vehicle = GroundVehicle {
        mass_kg: 980.0,
        angular_inertia_kgm2: Vec3::new(700.0, 840.0, 980.0),
        center_of_mass_offset: Vec3::new(0.0, -0.42, 0.0), // low CG for slope stability
        steering: SteeringConfig {
            max_angle_rad: 24.0_f32.to_radians(),
            steer_rate_rad_per_sec: 2.0,
            speed_reduction_start_mps: 6.0,
            speed_reduction_end_mps: 14.0,
            minimum_speed_factor: 0.65,
            ..default()
        },
        powertrain: PowertrainConfig {
            engine: EngineConfig {
                peak_torque_nm: 185.0,
                peak_torque_rpm: 2_600.0,
                redline_rpm: 4_500.0,
                idle_torque_fraction: 0.78, // high idle for crawling
                redline_torque_fraction: 0.72,
                engine_brake_torque_nm: 65.0,
                ..default()
            },
            gear_model: GearModel::Automatic(AutomaticGearboxConfig {
                final_drive_ratio: 6.10, // low gearing for torque
                forward_gears: [3.85, 2.35, 1.55, 1.12, 0.92, 0.78],
                forward_gear_count: 4,
                reverse_ratio: 3.45,
                shift_up_rpm: 4_050.0,
                shift_down_rpm: 2_050.0,
                coupling_speed_mps: 1.2,
                ..default()
            }),
            drive_model: DriveModel::Axle(AxleDriveConfig {
                differential: DifferentialConfig {
                    mode: DifferentialMode::LimitedSlip, // LSD for traction on slopes
                    limited_slip_load_bias: 0.62,
                },
                ..default()
            }),
            brake_force_newtons: 10_500.0,
            auxiliary_brake_force_newtons: 8_500.0,
            ..default()
        },
        stability: StabilityConfig {
            anti_roll_force_n_per_ratio: 4_400.0,
            park_hold_force_newtons: 12_000.0, // strong hill-hold
            park_hold_speed_threshold_mps: 1.6,
            low_speed_traction_boost: 1.6, // extra grip at crawl speed
            low_speed_traction_speed_threshold_mps: 2.4,
            yaw_stability_torque_nm_per_radps: 1_300.0,
            ..default()
        },
        aerodynamics: AerodynamicsConfig {
            drag_force_per_speed_sq: 0.8,
            downforce_per_speed_sq: 0.0, // off-road, no aero downforce
        },
    };

    let chassis_size = Vec3::new(1.75, 0.75, 3.25);
    // Spawn on the main ramp (centered at y=1.4, tilted -0.26 rad around X).
    // The ramp surface rises toward -Z, so z=-2.0 puts us partway up the slope.
    let transform = Transform::from_xyz(0.0, 3.6, -2.0).with_rotation(Quat::from_rotation_x(-0.26));

    // ---------------------------------------------------------------------------
    // Spawn chassis
    // ---------------------------------------------------------------------------
    let chassis_entity = commands
        .spawn((
            Name::new("Slope Rover"),
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
                base_color: Color::srgb(0.72, 0.78, 0.33),
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
            Name::new("Slope Rover Roof"),
            Mesh3d(meshes.add(Cuboid::new(
                chassis_size.x * 0.72,
                chassis_size.y * 0.42,
                chassis_size.z * 0.45,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.72, 0.78, 0.33).mix(&Color::WHITE, 0.18),
                perceptual_roughness: 0.46,
                ..default()
            })),
            Transform::from_xyz(0.0, chassis_size.y * 0.46, 0.12),
        ));
    });

    // ---------------------------------------------------------------------------
    // Long-travel suspension — high droop and compression for articulation
    // ---------------------------------------------------------------------------
    let suspension = SuspensionConfig {
        rest_length_m: 0.40,
        max_compression_m: 0.20,           // long travel
        max_droop_m: 0.18,                 // long droop
        spring_strength_n_per_m: 24_000.0, // softer for compliance
        damper_strength_n_per_mps: 3_000.0,
        bump_stop_strength_n_per_m: 16_000.0,
    };

    // ---------------------------------------------------------------------------
    // Off-road tires — high longitudinal grip, low-speed lateral boost
    // ---------------------------------------------------------------------------
    let tire = TireGripConfig {
        longitudinal_grip: 1.72, // aggressive tread
        lateral_grip: 1.08,
        low_speed_lateral_multiplier: 1.48, // extra grip at crawl speed
        nominal_load_newtons: 2_800.0,
        ..default()
    };

    // ---------------------------------------------------------------------------
    // Wheels — 4WD, front steered, rear driven + handbrake
    // ---------------------------------------------------------------------------
    #[rustfmt::skip]
    let wheel_specs: &[(u8, WheelSide, Vec3, f32, f32, f32, f32, f32, f32, f32)] = &[
        // axle, side,           mount_point,                   radius, width, inertia, steer, drive, brake, handbrake
        (0, WheelSide::Left,  Vec3::new(-0.78, -0.18, -0.95), 0.40, 0.26, 0.98, 1.0, 1.0, 1.0, 0.0),
        (0, WheelSide::Right, Vec3::new( 0.78, -0.18, -0.95), 0.40, 0.26, 0.98, 1.0, 1.0, 1.0, 0.0),
        (1, WheelSide::Left,  Vec3::new(-0.78, -0.18,  0.95), 0.40, 0.26, 1.02, 0.0, 1.0, 1.0, 1.0),
        (1, WheelSide::Right, Vec3::new( 0.78, -0.18,  0.95), 0.40, 0.26, 1.02, 0.0, 1.0, 1.0, 1.0),
    ];

    let wheel_color = Color::srgb(0.10, 0.10, 0.12);
    for (i, &(axle, side, mount, radius, width, inertia, steer, drive, brake, handbrake)) in
        wheel_specs.iter().enumerate()
    {
        let visual_entity = commands
            .spawn((
                Name::new(format!("Slope Rover Wheel Visual {}", i + 1)),
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
            Name::new(format!("Slope Rover Wheel {}", i + 1)),
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
                suspension,
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
