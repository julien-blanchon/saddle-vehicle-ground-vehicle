//! Drift tuning ground vehicle example — rear-wheel-drive coupe tuned for drifting.
//!
//! Shows tire grip, torque split, stability assist, and Magic Formula tire model
//! parameters for a purpose-built drift car.  WASD to steer/throttle, Space to
//! brake, Shift for auxiliary brake, R to reset.

use bevy::prelude::*;
use ground_vehicle::{
    AutomaticGearboxConfig, AxleDriveConfig, DifferentialConfig, DifferentialMode, DriveModel,
    EngineConfig, GearModel, GroundVehicle, GroundVehicleDriftConfig, GroundVehicleSurface,
    GroundVehicleWheel, GroundVehicleWheelVisual, MagicFormulaConfig, PowertrainConfig,
    StabilityConfig, SteeringConfig, SuspensionConfig, TireGripConfig, TireModel, VehicleIntent,
    WheelSide,
};
use ground_vehicle_example_support as support;
use support::{
    ExampleDriver, ResetPose, ScriptedControlOverride, driver_actions, spawn_overlay,
    spawn_surface_box, spawn_world,
};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle drift_tuning", true);
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
        "Drift Pad",
        Vec3::new(42.0, 0.06, 42.0),
        Transform::from_xyz(0.0, 0.03, 0.0),
        Color::srgb(0.13, 0.14, 0.16),
        GroundVehicleSurface {
            longitudinal_grip_scale: 0.98,
            lateral_grip_scale: 0.90,
            ..default()
        },
    );
    spawn_overlay(&mut commands, "ground_vehicle drift_tuning");

    // ---------------------------------------------------------------------------
    // Drift coupe — lightweight RWD with aggressive rear tire setup
    // ---------------------------------------------------------------------------
    let vehicle = GroundVehicle {
        mass_kg: 1_250.0,
        angular_inertia_kgm2: Vec3::new(600.0, 760.0, 980.0),
        steering: SteeringConfig {
            max_angle_rad: 40.0_f32.to_radians(), // wide lock for counter-steer
            steer_rate_rad_per_sec: 3.8,          // fast hands
            ..default()
        },
        powertrain: PowertrainConfig {
            engine: EngineConfig {
                peak_torque_nm: 410.0,
                peak_torque_rpm: 4_800.0,
                redline_rpm: 7_200.0,
                idle_torque_fraction: 0.40,
                redline_torque_fraction: 0.56,
                engine_brake_torque_nm: 95.0,
                ..default()
            },
            gear_model: GearModel::Automatic(AutomaticGearboxConfig {
                final_drive_ratio: 4.10,
                forward_gears: [3.12, 2.10, 1.55, 1.22, 1.00, 0.82],
                forward_gear_count: 5,
                reverse_ratio: 3.00,
                shift_up_rpm: 6_300.0,
                shift_down_rpm: 3_600.0,
                ..default()
            }),
            drive_model: DriveModel::Axle(AxleDriveConfig {
                differential: DifferentialConfig {
                    mode: DifferentialMode::Spool, // locked diff for easy kick-out
                    ..default()
                },
                ..default()
            }),
            auxiliary_brake_force_newtons: 11_200.0,
            ..default()
        },
        stability: StabilityConfig {
            yaw_stability_torque_nm_per_radps: 700.0, // mild stability
            ..default()
        },
        ..default()
    };

    let chassis_size = Vec3::new(1.90, 0.66, 4.35);
    let transform = Transform::from_xyz(0.0, 1.18, 16.0);

    // ---------------------------------------------------------------------------
    // Spawn chassis
    // ---------------------------------------------------------------------------
    let chassis_entity = commands
        .spawn((
            Name::new("Street Drift Coupe"),
            ExampleDriver,
            vehicle,
            (
                GroundVehicleDriftConfig {
                    entry_ratio: 0.22,
                    exit_ratio: 0.16,
                    ..default()
                },
                VehicleIntent::default(),
                ScriptedControlOverride::default(),
            ),
            (
                avian3d::prelude::Mass(vehicle.mass_kg),
                avian3d::prelude::AngularInertia::new(vehicle.angular_inertia_kgm2),
                avian3d::prelude::CenterOfMass::new(0.0, -0.35, 0.0),
            ),
            ResetPose {
                transform,
                linear_velocity: Vec3::ZERO,
                angular_velocity: Vec3::ZERO,
            },
            avian3d::prelude::Collider::cuboid(chassis_size.x, chassis_size.y, chassis_size.z),
            Mesh3d(meshes.add(Cuboid::new(chassis_size.x, chassis_size.y, chassis_size.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.10, 0.41, 0.83),
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
            Name::new("Street Drift Coupe Roof"),
            Mesh3d(meshes.add(Cuboid::new(
                chassis_size.x * 0.72,
                chassis_size.y * 0.42,
                chassis_size.z * 0.45,
            ))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.10, 0.41, 0.83).mix(&Color::WHITE, 0.18),
                perceptual_roughness: 0.46,
                ..default()
            })),
            Transform::from_xyz(0.0, chassis_size.y * 0.46, 0.12),
        ));
    });

    // ---------------------------------------------------------------------------
    // Suspension (shared base)
    // ---------------------------------------------------------------------------
    let base_suspension = SuspensionConfig {
        rest_length_m: 0.34,
        max_compression_m: 0.16,
        max_droop_m: 0.15,
        spring_strength_n_per_m: 28_000.0,
        damper_strength_n_per_mps: 3_500.0,
        bump_stop_strength_n_per_m: 18_000.0,
    };

    // ---------------------------------------------------------------------------
    // Front tires — standard linear model, good grip for counter-steering
    // ---------------------------------------------------------------------------
    let front_tire = TireGripConfig {
        longitudinal_grip: 1.55,
        lateral_grip: 1.12,
        ..default()
    };

    // ---------------------------------------------------------------------------
    // Rear tires — Magic Formula model, low lateral grip for easy slide
    // ---------------------------------------------------------------------------
    let rear_tire = TireGripConfig {
        model: TireModel::MagicFormula,
        longitudinal_grip: 1.18,
        lateral_grip: 0.82, // deliberately low for drift
        low_speed_slip_reference_mps: 1.8,
        auxiliary_brake_lateral_multiplier: 0.24, // massive lateral loss on auxiliary brake
        auxiliary_brake_longitudinal_multiplier: 0.12,
        magic_formula: MagicFormulaConfig {
            longitudinal_peak_slip_ratio: 0.16,
            lateral_peak_slip_angle_rad: 13.0_f32.to_radians(),
            ..default()
        },
        ..default()
    };

    // ---------------------------------------------------------------------------
    // Wheels — front: steered only, rear: driven only (RWD)
    // ---------------------------------------------------------------------------
    struct WheelDef {
        axle: u8,
        side: WheelSide,
        mount: Vec3,
        radius: f32,
        width: f32,
        inertia: f32,
        steer: f32,
        drive: f32,
        brake: f32,
        handbrake: f32,
        suspension: SuspensionConfig,
        tire: TireGripConfig,
    }

    let wheels = [
        // Front-left  (steered, not driven)
        WheelDef {
            axle: 0,
            side: WheelSide::Left,
            mount: Vec3::new(-0.82, -0.20, -1.24),
            radius: 0.36,
            width: 0.24,
            inertia: 1.02,
            steer: 1.0,
            drive: 0.0,
            brake: 1.0,
            handbrake: 0.0,
            suspension: base_suspension,
            tire: front_tire,
        },
        // Front-right (steered, not driven)
        WheelDef {
            axle: 0,
            side: WheelSide::Right,
            mount: Vec3::new(0.82, -0.20, -1.24),
            radius: 0.36,
            width: 0.24,
            inertia: 1.02,
            steer: 1.0,
            drive: 0.0,
            brake: 1.0,
            handbrake: 0.0,
            suspension: base_suspension,
            tire: front_tire,
        },
        // Rear-left   (driven, Magic Formula, handbrake)
        WheelDef {
            axle: 1,
            side: WheelSide::Left,
            mount: Vec3::new(-0.82, -0.20, 1.20),
            radius: 0.37,
            width: 0.26,
            inertia: 0.94,
            steer: 0.0,
            drive: 1.0,
            brake: 1.0,
            handbrake: 1.0,
            suspension: SuspensionConfig {
                spring_strength_n_per_m: 30_000.0,
                ..base_suspension
            },
            tire: rear_tire,
        },
        // Rear-right  (driven, Magic Formula, handbrake)
        WheelDef {
            axle: 1,
            side: WheelSide::Right,
            mount: Vec3::new(0.82, -0.20, 1.20),
            radius: 0.37,
            width: 0.26,
            inertia: 0.94,
            steer: 0.0,
            drive: 1.0,
            brake: 1.0,
            handbrake: 1.0,
            suspension: SuspensionConfig {
                spring_strength_n_per_m: 30_000.0,
                ..base_suspension
            },
            tire: rear_tire,
        },
    ];

    let wheel_color = Color::srgb(0.08, 0.08, 0.09);
    for (i, w) in wheels.into_iter().enumerate() {
        let visual_entity = commands
            .spawn((
                Name::new(format!("Street Drift Coupe Wheel Visual {}", i + 1)),
                Mesh3d(meshes.add(Cylinder::new(w.radius, w.width.max(0.08)))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: wheel_color,
                    perceptual_roughness: 0.92,
                    metallic: 0.02,
                    ..default()
                })),
                Transform::from_translation(transform.transform_point(w.mount)),
            ))
            .id();

        commands.spawn((
            Name::new(format!("Street Drift Coupe Wheel {}", i + 1)),
            GroundVehicleWheel {
                chassis: chassis_entity,
                axle: w.axle,
                side: w.side,
                drive_side: w.side,
                mount_point: w.mount,
                radius_m: w.radius,
                width_m: w.width,
                rotational_inertia_kgm2: w.inertia,
                steer_factor: w.steer,
                drive_factor: w.drive,
                brake_factor: w.brake,
                auxiliary_brake_factor: w.handbrake,
                suspension: w.suspension,
                tire: w.tire,
            },
            GroundVehicleWheelVisual {
                visual_entity,
                base_rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                ..default()
            },
        ));
    }
}
