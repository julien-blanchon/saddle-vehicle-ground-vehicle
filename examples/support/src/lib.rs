use avian3d::prelude::*;
use bevy::{
    prelude::*,
    window::{WindowPlugin, WindowResolution},
};
use bevy_enhanced_input::context::InputContextAppExt;
use bevy_enhanced_input::prelude::{
    Action, Bidirectional, Bindings, Cancel as InputCancel, Complete, ContextActivity, Fire,
    InputAction, Press as InputPress, Start, actions, bindings,
};
use ground_vehicle::{
    AerodynamicsConfig, DifferentialMode, DrivetrainConfig, GroundVehicle, GroundVehicleControl,
    GroundVehicleDebugDraw, GroundVehiclePlugin, GroundVehicleSurface, GroundVehicleTelemetry,
    GroundVehicleWheel, GroundVehicleWheelVisual, ReversePolicy, StabilityConfig, SteeringConfig,
    SteeringMode, SuspensionConfig, TireGripConfig, WheelSide,
};

#[derive(Resource, Clone, Copy)]
pub struct ExampleTitle(pub &'static str);

#[derive(Component)]
pub struct ExampleDriver;

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ScriptedControlOverride(pub Option<GroundVehicleControl>);

#[derive(Component)]
pub struct FollowCamera {
    pub distance: f32,
    pub height: f32,
    pub lateral_offset: f32,
}

#[derive(Component)]
pub struct OverlayText;

#[derive(Component, Clone, Copy)]
pub struct ResetPose {
    pub transform: Transform,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
}

#[derive(Debug, InputAction)]
#[action_output(f32)]
pub struct ThrottleAction;

#[derive(Debug, InputAction)]
#[action_output(f32)]
pub struct SteeringAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct BrakeAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct HandbrakeAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
pub struct ResetVehicleAction;

#[derive(Debug, Clone, Copy)]
pub struct WheelSpec {
    pub axle: u8,
    pub side: WheelSide,
    pub drive_side: WheelSide,
    pub mount_point: Vec3,
    pub radius_m: f32,
    pub width_m: f32,
    pub steer_factor: f32,
    pub drive_factor: f32,
    pub brake_factor: f32,
    pub handbrake_factor: f32,
    pub suspension: SuspensionConfig,
    pub tire: TireGripConfig,
}

impl WheelSpec {
    fn into_wheel(self, chassis: Entity) -> GroundVehicleWheel {
        GroundVehicleWheel {
            chassis,
            axle: self.axle,
            side: self.side,
            drive_side: self.drive_side,
            mount_point: self.mount_point,
            radius_m: self.radius_m,
            width_m: self.width_m,
            steer_factor: self.steer_factor,
            drive_factor: self.drive_factor,
            brake_factor: self.brake_factor,
            handbrake_factor: self.handbrake_factor,
            suspension: self.suspension,
            tire: self.tire,
        }
    }
}

pub fn configure_example_app(app: &mut App, title: &'static str, debug_draw: bool) {
    app.add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: title.into(),
                resolution: WindowResolution::new(1440, 900),
                ..default()
            }),
            ..default()
        }),
        PhysicsPlugins::default(),
        GroundVehiclePlugin::default(),
    ))
    .insert_resource(Time::<Fixed>::from_hz(60.0))
    .insert_resource(ExampleTitle(title));
    if !app.is_plugin_added::<bevy_enhanced_input::prelude::EnhancedInputPlugin>() {
        app.add_plugins(bevy_enhanced_input::prelude::EnhancedInputPlugin);
    }
    if debug_draw {
        app.insert_resource(GroundVehicleDebugDraw {
            enabled: true,
            ..default()
        });
    }
    app.add_input_context::<ExampleDriver>()
        .insert_resource(ClearColor(Color::srgb(0.57, 0.69, 0.84)))
        .add_observer(apply_throttle)
        .add_observer(clear_throttle_on_cancel)
        .add_observer(clear_throttle_on_complete)
        .add_observer(apply_steering)
        .add_observer(clear_steering_on_cancel)
        .add_observer(clear_steering_on_complete)
        .add_observer(apply_brake)
        .add_observer(clear_brake_on_cancel)
        .add_observer(clear_brake_on_complete)
        .add_observer(apply_handbrake)
        .add_observer(clear_handbrake_on_cancel)
        .add_observer(clear_handbrake_on_complete)
        .add_observer(reset_vehicle)
        .add_systems(
            Update,
            (
                apply_scripted_control_overrides,
                follow_camera,
                update_overlay,
            )
                .chain(),
        );
}

pub fn driver_actions() -> impl Bundle {
    actions!(ExampleDriver[
        (
            Action::<ThrottleAction>::new(),
            Bindings::spawn(Bidirectional::new(KeyCode::KeyS, KeyCode::KeyW)),
        ),
        (
            Action::<SteeringAction>::new(),
            Bindings::spawn(Bidirectional::new(KeyCode::KeyA, KeyCode::KeyD)),
        ),
        (
            Action::<BrakeAction>::new(),
            InputPress::default(),
            bindings![KeyCode::Space],
        ),
        (
            Action::<HandbrakeAction>::new(),
            InputPress::default(),
            bindings![KeyCode::ShiftLeft],
        ),
        (
            Action::<ResetVehicleAction>::new(),
            InputPress::default(),
            bindings![KeyCode::KeyR],
        ),
    ])
}

pub fn spawn_world(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    commands.spawn((
        Name::new("Example Sun"),
        DirectionalLight {
            illuminance: 22_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(18.0, 28.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Name::new("Example Fill"),
        PointLight {
            intensity: 65_000.0,
            range: 180.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-28.0, 16.0, 24.0),
    ));

    spawn_surface_box(
        commands,
        meshes,
        materials,
        "Base Ground",
        Vec3::new(280.0, 0.4, 280.0),
        Transform::from_xyz(0.0, -0.2, 0.0),
        Color::srgb(0.23, 0.33, 0.25),
        GroundVehicleSurface::default(),
    );

    commands.spawn((
        Name::new("Primary Camera"),
        Camera3d::default(),
        FollowCamera {
            distance: 11.5,
            height: 4.8,
            lateral_offset: 0.0,
        },
        Transform::from_xyz(-10.0, 7.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub fn spawn_overlay(commands: &mut Commands, title: &'static str) {
    commands.spawn((
        Name::new("Overlay"),
        OverlayText,
        Text::new(format!("{title}\n")),
        Node {
            position_type: PositionType::Absolute,
            left: px(18.0),
            top: px(16.0),
            ..default()
        },
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

pub fn spawn_surface_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    size: Vec3,
    transform: Transform,
    color: Color,
    surface: GroundVehicleSurface,
) -> Entity {
    commands
        .spawn((
            Name::new(name.to_string()),
            RigidBody::Static,
            surface,
            Collider::cuboid(size.x, size.y, size.z),
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.92,
                ..default()
            })),
            transform,
        ))
        .id()
}

pub fn spawn_ramp(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    size: Vec3,
    transform: Transform,
    color: Color,
    surface: GroundVehicleSurface,
) -> Entity {
    spawn_surface_box(
        commands, meshes, materials, name, size, transform, color, surface,
    )
}

pub fn spawn_bump_strip(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    prefix: &str,
    start: Vec3,
    count: usize,
    spacing: f32,
) {
    for index in 0..count {
        let z = start.z - spacing * index as f32;
        spawn_surface_box(
            commands,
            meshes,
            materials,
            &format!("{prefix} {}", index + 1),
            Vec3::new(1.2, 0.28, 1.6),
            Transform::from_xyz(start.x, start.y, z),
            Color::srgb(0.43, 0.35, 0.21),
            GroundVehicleSurface::default(),
        );
    }
}

pub fn spawn_compact_car_demo(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    transform: Transform,
    pilot_controlled: bool,
) -> Entity {
    let vehicle = GroundVehicle {
        steering: SteeringConfig {
            max_angle_rad: 29.0_f32.to_radians(),
            ..default()
        },
        drivetrain: DrivetrainConfig {
            max_drive_force_newtons: 9_200.0,
            max_reverse_force_newtons: 4_500.0,
            brake_force_newtons: 15_000.0,
            differential: DifferentialMode::Open,
            ..default()
        },
        ..default()
    };
    spawn_vehicle(
        commands,
        meshes,
        materials,
        name,
        vehicle,
        Vec3::new(1.85, 0.72, 4.20),
        compact_car_wheels(),
        transform,
        Color::srgb(0.82, 0.21, 0.19),
        Color::srgb(0.12, 0.12, 0.13),
        pilot_controlled,
    )
}

pub fn spawn_drift_coupe_demo(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    transform: Transform,
    pilot_controlled: bool,
) -> Entity {
    let vehicle = GroundVehicle {
        mass_kg: 1_250.0,
        angular_inertia_kgm2: Vec3::new(600.0, 760.0, 980.0),
        steering: SteeringConfig {
            max_angle_rad: 40.0_f32.to_radians(),
            steer_rate_rad_per_sec: 3.8,
            ..default()
        },
        drivetrain: DrivetrainConfig {
            max_drive_force_newtons: 10_400.0,
            handbrake_force_newtons: 11_200.0,
            differential: DifferentialMode::Spool,
            ..default()
        },
        stability: StabilityConfig {
            yaw_stability_torque_nm_per_radps: 700.0,
            drift_entry_ratio: 0.22,
            drift_exit_ratio: 0.16,
            ..default()
        },
        ..default()
    };
    spawn_vehicle(
        commands,
        meshes,
        materials,
        name,
        vehicle,
        Vec3::new(1.90, 0.66, 4.35),
        drift_coupe_wheels(),
        transform,
        Color::srgb(0.10, 0.41, 0.83),
        Color::srgb(0.08, 0.08, 0.09),
        pilot_controlled,
    )
}

pub fn spawn_cargo_truck_demo(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    transform: Transform,
    pilot_controlled: bool,
) -> Entity {
    let vehicle = cargo_truck_vehicle();
    spawn_vehicle(
        commands,
        meshes,
        materials,
        name,
        vehicle,
        Vec3::new(2.35, 1.05, 7.80),
        cargo_truck_wheels(),
        transform,
        Color::srgb(0.72, 0.56, 0.23),
        Color::srgb(0.10, 0.10, 0.11),
        pilot_controlled,
    )
}

pub fn spawn_skid_vehicle_demo(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    transform: Transform,
    pilot_controlled: bool,
) -> Entity {
    let vehicle = skid_vehicle();
    spawn_vehicle(
        commands,
        meshes,
        materials,
        name,
        vehicle,
        Vec3::new(2.30, 1.05, 4.90),
        skid_vehicle_wheels(),
        transform,
        Color::srgb(0.28, 0.52, 0.35),
        Color::srgb(0.07, 0.07, 0.08),
        pilot_controlled,
    )
}

pub fn spawn_rover_demo(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    transform: Transform,
    pilot_controlled: bool,
) -> Entity {
    let vehicle = rover_vehicle();
    spawn_vehicle(
        commands,
        meshes,
        materials,
        name,
        vehicle,
        Vec3::new(1.75, 0.75, 3.25),
        rover_wheels(),
        transform,
        Color::srgb(0.72, 0.78, 0.33),
        Color::srgb(0.10, 0.10, 0.12),
        pilot_controlled,
    )
}

pub fn set_camera_preset(
    camera: &mut Query<&mut FollowCamera>,
    distance: f32,
    height: f32,
    lateral: f32,
) {
    if let Ok(mut follow) = camera.single_mut() {
        follow.distance = distance;
        follow.height = height;
        follow.lateral_offset = lateral;
    }
}

fn spawn_vehicle(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    vehicle: GroundVehicle,
    chassis_size: Vec3,
    wheels: Vec<WheelSpec>,
    transform: Transform,
    body_color: Color,
    wheel_color: Color,
    pilot_controlled: bool,
) -> Entity {
    let mut chassis = commands.spawn((
        Name::new(name.to_string()),
        ExampleDriver,
        vehicle,
        GroundVehicleControl::default(),
        ScriptedControlOverride::default(),
        Mass(vehicle.mass_kg),
        AngularInertia::new(vehicle.angular_inertia_kgm2),
        CenterOfMass::new(
            vehicle.center_of_mass_offset.x,
            vehicle.center_of_mass_offset.y,
            vehicle.center_of_mass_offset.z,
        ),
        ResetPose {
            transform,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
        },
        Collider::cuboid(chassis_size.x, chassis_size.y, chassis_size.z),
        Mesh3d(meshes.add(Cuboid::new(chassis_size.x, chassis_size.y, chassis_size.z))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: body_color,
            perceptual_roughness: 0.58,
            metallic: 0.08,
            ..default()
        })),
        transform,
    ));
    if pilot_controlled {
        chassis.insert((driver_actions(), ContextActivity::<ExampleDriver>::ACTIVE));
    } else {
        chassis.insert(ContextActivity::<ExampleDriver>::INACTIVE);
    }
    let chassis_entity = chassis.id();

    let roof_height = chassis_size.y * 0.46;
    commands.spawn((
        Name::new(format!("{name} Roof")),
        Mesh3d(meshes.add(Cuboid::new(
            chassis_size.x * 0.72,
            chassis_size.y * 0.42,
            chassis_size.z * 0.45,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: body_color.mix(&Color::WHITE, 0.18),
            perceptual_roughness: 0.46,
            ..default()
        })),
        Transform::from_translation(
            transform.translation + transform.rotation * Vec3::new(0.0, roof_height, 0.12),
        )
        .with_rotation(transform.rotation),
    ));

    for (index, wheel_spec) in wheels.into_iter().enumerate() {
        let visual_entity = commands
            .spawn((
                Name::new(format!("{name} Wheel Visual {}", index + 1)),
                Mesh3d(meshes.add(Cylinder::new(
                    wheel_spec.radius_m,
                    wheel_spec.width_m.max(0.08),
                ))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: wheel_color,
                    perceptual_roughness: 0.92,
                    metallic: 0.02,
                    ..default()
                })),
                Transform::from_translation(transform.transform_point(wheel_spec.mount_point)),
            ))
            .id();

        commands.spawn((
            Name::new(format!("{name} Wheel {}", index + 1)),
            wheel_spec.into_wheel(chassis_entity),
            GroundVehicleWheelVisual {
                visual_entity,
                base_rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                ..default()
            },
        ));
    }

    chassis_entity
}

fn compact_car_wheels() -> Vec<WheelSpec> {
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
    vec![
        WheelSpec {
            axle: 0,
            side: WheelSide::Left,
            drive_side: WheelSide::Left,
            mount_point: Vec3::new(-0.82, -0.20, -1.24),
            radius_m: 0.36,
            width_m: 0.24,
            steer_factor: 1.0,
            drive_factor: 1.0,
            brake_factor: 1.0,
            handbrake_factor: 0.0,
            suspension: front_suspension,
            tire: front_tire,
        },
        WheelSpec {
            axle: 0,
            side: WheelSide::Right,
            drive_side: WheelSide::Right,
            mount_point: Vec3::new(0.82, -0.20, -1.24),
            ..WheelSpec {
                axle: 0,
                side: WheelSide::Left,
                drive_side: WheelSide::Left,
                mount_point: Vec3::new(-0.82, -0.20, -1.24),
                radius_m: 0.36,
                width_m: 0.24,
                steer_factor: 1.0,
                drive_factor: 1.0,
                brake_factor: 1.0,
                handbrake_factor: 0.0,
                suspension: front_suspension,
                tire: front_tire,
            }
        },
        WheelSpec {
            axle: 1,
            side: WheelSide::Left,
            drive_side: WheelSide::Left,
            mount_point: Vec3::new(-0.82, -0.20, 1.20),
            radius_m: 0.37,
            width_m: 0.26,
            steer_factor: 0.0,
            drive_factor: 0.0,
            brake_factor: 1.0,
            handbrake_factor: 1.0,
            suspension: rear_suspension,
            tire: rear_tire,
        },
        WheelSpec {
            axle: 1,
            side: WheelSide::Right,
            drive_side: WheelSide::Right,
            mount_point: Vec3::new(0.82, -0.20, 1.20),
            ..WheelSpec {
                axle: 1,
                side: WheelSide::Left,
                drive_side: WheelSide::Left,
                mount_point: Vec3::new(-0.82, -0.20, 1.20),
                radius_m: 0.37,
                width_m: 0.26,
                steer_factor: 0.0,
                drive_factor: 0.0,
                brake_factor: 1.0,
                handbrake_factor: 1.0,
                suspension: rear_suspension,
                tire: rear_tire,
            }
        },
    ]
}

fn drift_coupe_wheels() -> Vec<WheelSpec> {
    let mut wheels = compact_car_wheels();
    for wheel in &mut wheels {
        if wheel.axle == 0 {
            wheel.drive_factor = 0.0;
            wheel.steer_factor = 1.0;
            wheel.tire.lateral_grip = 1.12;
        } else {
            wheel.drive_factor = 1.0;
            wheel.handbrake_factor = 1.0;
            wheel.tire.lateral_grip = 0.82;
            wheel.tire.handbrake_lateral_multiplier = 0.24;
            wheel.tire.handbrake_longitudinal_multiplier = 0.12;
        }
    }
    wheels
}

fn cargo_truck_vehicle() -> GroundVehicle {
    GroundVehicle {
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
            differential: DifferentialMode::LimitedSlip,
            max_drive_force_newtons: 18_000.0,
            max_reverse_force_newtons: 9_000.0,
            brake_force_newtons: 24_000.0,
            handbrake_force_newtons: 16_000.0,
            reverse_policy: ReversePolicy::StopThenReverse,
            limited_slip_load_bias: 0.68,
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
    }
}

fn cargo_truck_wheels() -> Vec<WheelSpec> {
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
    vec![
        WheelSpec {
            axle: 0,
            side: WheelSide::Left,
            drive_side: WheelSide::Left,
            mount_point: Vec3::new(-1.08, -0.35, -2.70),
            radius_m: 0.52,
            width_m: 0.34,
            steer_factor: 1.0,
            drive_factor: 0.0,
            brake_factor: 1.0,
            handbrake_factor: 0.0,
            suspension,
            tire,
        },
        WheelSpec {
            axle: 0,
            side: WheelSide::Right,
            drive_side: WheelSide::Right,
            mount_point: Vec3::new(1.08, -0.35, -2.70),
            ..WheelSpec {
                axle: 0,
                side: WheelSide::Left,
                drive_side: WheelSide::Left,
                mount_point: Vec3::new(-1.08, -0.35, -2.70),
                radius_m: 0.52,
                width_m: 0.34,
                steer_factor: 1.0,
                drive_factor: 0.0,
                brake_factor: 1.0,
                handbrake_factor: 0.0,
                suspension,
                tire,
            }
        },
        WheelSpec {
            axle: 1,
            side: WheelSide::Left,
            drive_side: WheelSide::Left,
            mount_point: Vec3::new(-1.12, -0.35, 0.15),
            radius_m: 0.54,
            width_m: 0.36,
            steer_factor: 0.0,
            drive_factor: 1.0,
            brake_factor: 1.0,
            handbrake_factor: 0.5,
            suspension,
            tire,
        },
        WheelSpec {
            axle: 1,
            side: WheelSide::Right,
            drive_side: WheelSide::Right,
            mount_point: Vec3::new(1.12, -0.35, 0.15),
            ..WheelSpec {
                axle: 1,
                side: WheelSide::Left,
                drive_side: WheelSide::Left,
                mount_point: Vec3::new(-1.12, -0.35, 0.15),
                radius_m: 0.54,
                width_m: 0.36,
                steer_factor: 0.0,
                drive_factor: 1.0,
                brake_factor: 1.0,
                handbrake_factor: 0.5,
                suspension,
                tire,
            }
        },
        WheelSpec {
            axle: 2,
            side: WheelSide::Left,
            drive_side: WheelSide::Left,
            mount_point: Vec3::new(-1.12, -0.35, 2.55),
            radius_m: 0.54,
            width_m: 0.36,
            steer_factor: 0.0,
            drive_factor: 1.0,
            brake_factor: 1.0,
            handbrake_factor: 1.0,
            suspension,
            tire,
        },
        WheelSpec {
            axle: 2,
            side: WheelSide::Right,
            drive_side: WheelSide::Right,
            mount_point: Vec3::new(1.12, -0.35, 2.55),
            ..WheelSpec {
                axle: 2,
                side: WheelSide::Left,
                drive_side: WheelSide::Left,
                mount_point: Vec3::new(-1.12, -0.35, 2.55),
                radius_m: 0.54,
                width_m: 0.36,
                steer_factor: 0.0,
                drive_factor: 1.0,
                brake_factor: 1.0,
                handbrake_factor: 1.0,
                suspension,
                tire,
            }
        },
    ]
}

fn skid_vehicle() -> GroundVehicle {
    GroundVehicle {
        mass_kg: 2_100.0,
        angular_inertia_kgm2: Vec3::new(1_800.0, 2_600.0, 3_300.0),
        center_of_mass_offset: Vec3::new(0.0, -0.45, 0.0),
        steering: SteeringConfig {
            mode: SteeringMode::SkidSteer,
            skid_steer_turn_scale: 0.92,
            max_angle_rad: 0.0,
            ackermann_ratio: 0.0,
            minimum_speed_factor: 1.0,
            ..default()
        },
        drivetrain: DrivetrainConfig {
            differential: DifferentialMode::Spool,
            max_drive_force_newtons: 13_500.0,
            max_reverse_force_newtons: 8_500.0,
            brake_force_newtons: 15_000.0,
            handbrake_force_newtons: 6_000.0,
            reverse_policy: ReversePolicy::Immediate,
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
    }
}

fn skid_vehicle_wheels() -> Vec<WheelSpec> {
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
    let z_positions = [-1.7, 0.0, 1.7];
    let mut wheels = Vec::new();
    for (axle, z) in z_positions.into_iter().enumerate() {
        wheels.push(WheelSpec {
            axle: axle as u8,
            side: WheelSide::Left,
            drive_side: WheelSide::Left,
            mount_point: Vec3::new(-0.95, -0.28, z),
            radius_m: 0.42,
            width_m: 0.28,
            steer_factor: 0.0,
            drive_factor: 1.0,
            brake_factor: 1.0,
            handbrake_factor: 0.6,
            suspension,
            tire,
        });
        wheels.push(WheelSpec {
            axle: axle as u8,
            side: WheelSide::Right,
            drive_side: WheelSide::Right,
            mount_point: Vec3::new(0.95, -0.28, z),
            radius_m: 0.42,
            width_m: 0.28,
            steer_factor: 0.0,
            drive_factor: 1.0,
            brake_factor: 1.0,
            handbrake_factor: 0.6,
            suspension,
            tire,
        });
    }
    wheels
}

fn rover_vehicle() -> GroundVehicle {
    GroundVehicle {
        mass_kg: 980.0,
        angular_inertia_kgm2: Vec3::new(700.0, 840.0, 980.0),
        center_of_mass_offset: Vec3::new(0.0, -0.42, 0.0),
        steering: SteeringConfig {
            max_angle_rad: 24.0_f32.to_radians(),
            steer_rate_rad_per_sec: 2.0,
            speed_reduction_start_mps: 6.0,
            speed_reduction_end_mps: 14.0,
            minimum_speed_factor: 0.65,
            ..default()
        },
        drivetrain: DrivetrainConfig {
            max_drive_force_newtons: 4_800.0,
            max_reverse_force_newtons: 3_800.0,
            brake_force_newtons: 10_500.0,
            handbrake_force_newtons: 8_500.0,
            differential: DifferentialMode::LimitedSlip,
            ..default()
        },
        stability: StabilityConfig {
            anti_roll_force_n_per_ratio: 4_400.0,
            park_hold_force_newtons: 12_000.0,
            park_hold_speed_threshold_mps: 1.6,
            low_speed_traction_boost: 1.6,
            low_speed_traction_speed_threshold_mps: 2.4,
            yaw_stability_torque_nm_per_radps: 1_300.0,
            ..default()
        },
        aerodynamics: AerodynamicsConfig {
            drag_force_per_speed_sq: 0.8,
            downforce_per_speed_sq: 0.0,
        },
    }
}

fn rover_wheels() -> Vec<WheelSpec> {
    let suspension = SuspensionConfig {
        rest_length_m: 0.40,
        max_compression_m: 0.20,
        max_droop_m: 0.18,
        spring_strength_n_per_m: 24_000.0,
        damper_strength_n_per_mps: 3_000.0,
        bump_stop_strength_n_per_m: 16_000.0,
    };
    let tire = TireGripConfig {
        longitudinal_grip: 1.72,
        lateral_grip: 1.08,
        low_speed_lateral_multiplier: 1.48,
        nominal_load_newtons: 2_800.0,
        ..default()
    };
    vec![
        WheelSpec {
            axle: 0,
            side: WheelSide::Left,
            drive_side: WheelSide::Left,
            mount_point: Vec3::new(-0.78, -0.18, -0.95),
            radius_m: 0.40,
            width_m: 0.26,
            steer_factor: 1.0,
            drive_factor: 1.0,
            brake_factor: 1.0,
            handbrake_factor: 0.0,
            suspension,
            tire,
        },
        WheelSpec {
            axle: 0,
            side: WheelSide::Right,
            drive_side: WheelSide::Right,
            mount_point: Vec3::new(0.78, -0.18, -0.95),
            radius_m: 0.40,
            width_m: 0.26,
            steer_factor: 1.0,
            drive_factor: 1.0,
            brake_factor: 1.0,
            handbrake_factor: 0.0,
            suspension,
            tire,
        },
        WheelSpec {
            axle: 1,
            side: WheelSide::Left,
            drive_side: WheelSide::Left,
            mount_point: Vec3::new(-0.78, -0.18, 0.95),
            radius_m: 0.40,
            width_m: 0.26,
            steer_factor: 0.0,
            drive_factor: 1.0,
            brake_factor: 1.0,
            handbrake_factor: 1.0,
            suspension,
            tire,
        },
        WheelSpec {
            axle: 1,
            side: WheelSide::Right,
            drive_side: WheelSide::Right,
            mount_point: Vec3::new(0.78, -0.18, 0.95),
            radius_m: 0.40,
            width_m: 0.26,
            steer_factor: 0.0,
            drive_factor: 1.0,
            brake_factor: 1.0,
            handbrake_factor: 1.0,
            suspension,
            tire,
        },
    ]
}

fn update_overlay(
    title: Res<ExampleTitle>,
    mut overlay: Query<&mut Text, With<OverlayText>>,
    active_driver: Query<
        (
            &Name,
            &GroundVehicleControl,
            &GroundVehicleTelemetry,
            Option<&ContextActivity<ExampleDriver>>,
        ),
        With<ExampleDriver>,
    >,
) {
    let Ok(mut overlay) = overlay.single_mut() else {
        return;
    };
    let Some((name, control, telemetry, _)) = active_driver
        .iter()
        .find(|(_, _, _, activity)| activity.is_none_or(|activity| **activity))
    else {
        overlay.0 = format!("{}\nNo active driver", title.0);
        return;
    };

    overlay.0 = format!(
        "{}\nActive vehicle: {}\nSpeed {:>6.1} m/s  Forward {:>6.1}  Lateral {:>5.1}\nGrounded wheels: {:>2}  Drift ratio {:>4.2}  Drifting {}\nSteer {:>4.2}  Throttle {:>4.2}  Brake {:>4.2}  Handbrake {:>4.2}\nControls: WASD steer/throttle, Space brake, Shift handbrake, R reset.",
        title.0,
        name.as_str(),
        telemetry.speed_mps,
        telemetry.forward_speed_mps,
        telemetry.lateral_speed_mps,
        telemetry.grounded_wheels,
        telemetry.drift_ratio,
        telemetry.drifting,
        control.steering,
        control.throttle,
        control.brake,
        control.handbrake,
    );
}

fn apply_scripted_control_overrides(
    mut vehicles: Query<(&mut GroundVehicleControl, &ScriptedControlOverride)>,
) {
    for (mut control, scripted_override) in &mut vehicles {
        if let Some(scripted) = scripted_override.0 {
            *control = scripted;
        }
    }
}

fn follow_camera(
    time: Res<Time>,
    target: Query<(&Transform, Option<&ContextActivity<ExampleDriver>>), With<ExampleDriver>>,
    mut camera: Query<(&FollowCamera, &mut Transform), (With<Camera3d>, Without<ExampleDriver>)>,
) {
    let Some(target_transform) = target.iter().find_map(|(transform, activity)| {
        activity
            .is_none_or(|activity| **activity)
            .then_some(transform)
    }) else {
        return;
    };
    let Ok((follow, mut camera_transform)) = camera.single_mut() else {
        return;
    };

    let desired = target_transform.translation - target_transform.forward() * follow.distance
        + Vec3::Y * follow.height
        + target_transform.right() * follow.lateral_offset;
    let alpha = (time.delta_secs() * 4.5).clamp(0.0, 1.0);
    camera_transform.translation = camera_transform.translation.lerp(desired, alpha);
    camera_transform.look_at(target_transform.translation + Vec3::Y * 1.1, Vec3::Y);
}

fn apply_throttle(
    trigger: On<Fire<ThrottleAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.throttle = trigger.value;
    }
}

fn clear_throttle_on_cancel(
    trigger: On<InputCancel<ThrottleAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.throttle = 0.0;
    }
}

fn clear_throttle_on_complete(
    trigger: On<Complete<ThrottleAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.throttle = 0.0;
    }
}

fn apply_steering(
    trigger: On<Fire<SteeringAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.steering = trigger.value;
    }
}

fn clear_steering_on_cancel(
    trigger: On<InputCancel<SteeringAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.steering = 0.0;
    }
}

fn clear_steering_on_complete(
    trigger: On<Complete<SteeringAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.steering = 0.0;
    }
}

fn apply_brake(
    trigger: On<Fire<BrakeAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.brake = f32::from(trigger.value);
    }
}

fn clear_brake_on_cancel(
    trigger: On<InputCancel<BrakeAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.brake = 0.0;
    }
}

fn clear_brake_on_complete(
    trigger: On<Complete<BrakeAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.brake = 0.0;
    }
}

fn apply_handbrake(
    trigger: On<Fire<HandbrakeAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.handbrake = f32::from(trigger.value);
    }
}

fn clear_handbrake_on_cancel(
    trigger: On<InputCancel<HandbrakeAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.handbrake = 0.0;
    }
}

fn clear_handbrake_on_complete(
    trigger: On<Complete<HandbrakeAction>>,
    mut controls: Query<&mut GroundVehicleControl, With<ExampleDriver>>,
) {
    if let Ok(mut control) = controls.get_mut(trigger.context) {
        control.handbrake = 0.0;
    }
}

fn reset_vehicle(
    trigger: On<Start<ResetVehicleAction>>,
    mut query: Query<
        (
            &ResetPose,
            &mut Transform,
            &mut LinearVelocity,
            &mut AngularVelocity,
            &mut GroundVehicleControl,
            &mut ScriptedControlOverride,
        ),
        With<ExampleDriver>,
    >,
) {
    let Ok((
        pose,
        mut transform,
        mut linear_velocity,
        mut angular_velocity,
        mut control,
        mut scripted_override,
    )) = query.get_mut(trigger.context)
    else {
        return;
    };
    *transform = pose.transform;
    *linear_velocity = LinearVelocity(pose.linear_velocity);
    *angular_velocity = AngularVelocity(pose.angular_velocity);
    *control = GroundVehicleControl::default();
    scripted_override.0 = None;
}
