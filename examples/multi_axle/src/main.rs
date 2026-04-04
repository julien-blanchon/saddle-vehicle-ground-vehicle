//! Multi-axle ground vehicle example — 3-axle cargo truck.
//!
//! Demonstrates explicit per-axle suspension, tire, and drive configuration for
//! a heavy 6-wheel truck.  WASD to steer/throttle, Space to brake, Shift for
//! handbrake, R to reset.

use bevy::prelude::*;
use ground_vehicle_example_support as support;
use ground_vehicle::{
    AerodynamicsConfig, DifferentialConfig, DifferentialMode, DrivetrainConfig, EngineConfig,
    GroundVehicle, GroundVehicleControl, GroundVehicleWheel,
    GroundVehicleWheelVisual, ReversePolicy, StabilityConfig, SteeringConfig, SuspensionConfig,
    TireGripConfig, TransmissionConfig, WheelSide,
};
use support::{
    ExampleDriver, ResetPose, ScriptedControlOverride, driver_actions, spawn_bump_strip,
    spawn_overlay, spawn_world,
};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle multi_axle", false);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);
    spawn_bump_strip(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Truck Bump",
        Vec3::new(0.0, 0.14, 10.0),
        6,
        3.0,
    );
    spawn_overlay(&mut commands, "ground_vehicle multi_axle");

    // ---------------------------------------------------------------------------
    // Heavy cargo truck — 4 800 kg, 3 axles (front steer, 2 rear driven)
    // ---------------------------------------------------------------------------
    let vehicle = GroundVehicle {
        mass_kg: 4_800.0,
        angular_inertia_kgm2: Vec3::new(5_000.0, 6_600.0, 8_400.0),
        center_of_mass_offset: Vec3::new(0.0, -0.55, 0.0),
        steering: SteeringConfig {
            max_angle_rad: 22.0_f32.to_radians(),
            steer_rate_rad_per_sec: 1.6,
            minimum_speed_factor: 0.45,
            speed_reduction_start_mps: 10.0,
            speed_reduction_end_mps: 24.0,
            ..default()
        },
        drivetrain: DrivetrainConfig {
            engine: EngineConfig {
                peak_torque_nm: 1_060.0,
                peak_torque_rpm: 1_900.0,
                redline_rpm: 3_400.0,
                idle_torque_fraction: 0.58,
                redline_torque_fraction: 0.48,
                engine_brake_torque_nm: 280.0,
                ..default()
            },
            transmission: TransmissionConfig {
                final_drive_ratio: 4.85,
                forward_gears: [6.40, 3.55, 2.35, 1.58, 1.22, 0.92],
                forward_gear_count: 6,
                reverse_ratio: 6.10,
                shift_up_rpm: 2_950.0,
                shift_down_rpm: 1_450.0,
                clutch_coupling_speed_mps: 2.4,
                ..default()
            },
            differential: DifferentialConfig {
                mode: DifferentialMode::LimitedSlip,
                limited_slip_load_bias: 0.68,
            },
            brake_force_newtons: 24_000.0,
            handbrake_force_newtons: 16_000.0,
            reverse_policy: ReversePolicy::StopThenReverse,
            ..default()
        },
        stability: StabilityConfig {
            anti_roll_force_n_per_ratio: 5_000.0,
            park_hold_force_newtons: 9_000.0,
            low_speed_traction_boost: 1.35,
            yaw_stability_torque_nm_per_radps: 3_200.0,
            airborne_upright_torque_nm_per_rad: 600.0,
            ..default()
        },
        aerodynamics: AerodynamicsConfig {
            drag_force_per_speed_sq: 1.8,
            downforce_per_speed_sq: 0.08,
        },
    };

    let chassis_size = Vec3::new(2.35, 1.05, 7.80);
    let transform = Transform::from_xyz(0.0, 1.65, 20.0);

    // ---------------------------------------------------------------------------
    // Spawn chassis
    // ---------------------------------------------------------------------------
    let chassis_entity = commands
        .spawn((
            Name::new("Cargo Truck"),
            ExampleDriver,
            vehicle,
            GroundVehicleControl::default(),
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
                base_color: Color::srgb(0.72, 0.56, 0.23),
                perceptual_roughness: 0.58,
                metallic: 0.08,
                ..default()
            })),
            transform,
            driver_actions(),
            bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::ACTIVE,
        ))
        .id();

    // Decorative roof / cab
    commands.spawn((
        Name::new("Cargo Truck Roof"),
        Mesh3d(meshes.add(Cuboid::new(
            chassis_size.x * 0.72,
            chassis_size.y * 0.42,
            chassis_size.z * 0.45,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.72, 0.56, 0.23).mix(&Color::WHITE, 0.18),
            perceptual_roughness: 0.46,
            ..default()
        })),
        Transform::from_translation(
            transform.translation
                + transform.rotation * Vec3::new(0.0, chassis_size.y * 0.46, 0.12),
        )
        .with_rotation(transform.rotation),
    ));

    // ---------------------------------------------------------------------------
    // Suspension & tire — shared heavy-duty spec across all axles
    // ---------------------------------------------------------------------------
    let suspension = SuspensionConfig {
        rest_length_m: 0.46,
        max_compression_m: 0.22,
        max_droop_m: 0.20,
        spring_strength_n_per_m: 52_000.0,
        damper_strength_n_per_mps: 5_200.0,
        bump_stop_strength_n_per_m: 28_000.0,
    };
    let tire = TireGripConfig {
        longitudinal_grip: 1.45,
        lateral_grip: 1.08,
        nominal_load_newtons: 8_500.0,
        load_sensitivity: 0.55,
        rolling_resistance_force_newtons: 48.0,
        ..default()
    };

    // ---------------------------------------------------------------------------
    // Wheels — 3 axles, 6 wheels total
    //   Axle 0 (front):  steered, not driven
    //   Axle 1 (mid-rear): driven, braked, partial handbrake
    //   Axle 2 (rear):     driven, braked, full handbrake
    // ---------------------------------------------------------------------------
    #[rustfmt::skip]
    let wheel_specs: &[(u8, WheelSide, Vec3, f32, f32, f32, f32, f32, f32, f32)] = &[
        // axle, side,           mount_point,                    radius, width, inertia, steer, drive, brake, handbrake
        (0, WheelSide::Left,  Vec3::new(-1.08, -0.35, -2.70), 0.52, 0.34, 2.35, 1.0, 0.0, 1.0, 0.0),
        (0, WheelSide::Right, Vec3::new( 1.08, -0.35, -2.70), 0.52, 0.34, 2.35, 1.0, 0.0, 1.0, 0.0),
        (1, WheelSide::Left,  Vec3::new(-1.12, -0.35,  0.15), 0.54, 0.36, 2.55, 0.0, 1.0, 1.0, 0.5),
        (1, WheelSide::Right, Vec3::new( 1.12, -0.35,  0.15), 0.54, 0.36, 2.55, 0.0, 1.0, 1.0, 0.5),
        (2, WheelSide::Left,  Vec3::new(-1.12, -0.35,  2.55), 0.54, 0.36, 2.55, 0.0, 1.0, 1.0, 1.0),
        (2, WheelSide::Right, Vec3::new( 1.12, -0.35,  2.55), 0.54, 0.36, 2.55, 0.0, 1.0, 1.0, 1.0),
    ];

    let wheel_color = Color::srgb(0.10, 0.10, 0.11);
    for (i, &(axle, side, mount, radius, width, inertia, steer, drive, brake, handbrake)) in wheel_specs.iter().enumerate() {
        let visual_entity = commands
            .spawn((
                Name::new(format!("Cargo Truck Wheel Visual {}", i + 1)),
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
            Name::new(format!("Cargo Truck Wheel {}", i + 1)),
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
                handbrake_factor: handbrake,
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
