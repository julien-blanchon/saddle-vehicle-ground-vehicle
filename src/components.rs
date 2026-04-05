use crate::config::{
    AerodynamicsConfig, DifferentialConfig, DrivetrainConfig, EngineConfig, StabilityConfig,
    SteeringConfig, SuspensionConfig, TireGripConfig, TransmissionConfig,
};
use avian3d::prelude::*;
use bevy::prelude::*;

const DEFAULT_MASS_KG: f32 = 1_300.0;
const DEFAULT_COM_OFFSET: Vec3 = Vec3::new(0.0, -0.40, 0.0);
const DEFAULT_INERTIA: Vec3 = Vec3::new(800.0, 1_000.0, 1_200.0);

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WheelSide {
    Left,
    Right,
    Center,
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component, Debug)]
#[require(
    AngularDamping = AngularDamping(0.85),
    AngularVelocity = AngularVelocity::ZERO,
    GroundVehicleControl,
    GroundVehicleInternal,
    GroundVehicleResolvedControl,
    GroundVehicleTelemetry,
    LinearDamping = LinearDamping(0.02),
    LinearVelocity = LinearVelocity::ZERO,
    RigidBody = RigidBody::Dynamic,
    Transform,
    GlobalTransform
)]
pub struct GroundVehicle {
    pub mass_kg: f32,
    pub angular_inertia_kgm2: Vec3,
    pub center_of_mass_offset: Vec3,
    pub steering: SteeringConfig,
    pub drivetrain: DrivetrainConfig,
    pub stability: StabilityConfig,
    pub aerodynamics: AerodynamicsConfig,
}

impl Default for GroundVehicle {
    fn default() -> Self {
        Self {
            mass_kg: DEFAULT_MASS_KG,
            angular_inertia_kgm2: DEFAULT_INERTIA,
            center_of_mass_offset: DEFAULT_COM_OFFSET,
            steering: SteeringConfig::default(),
            drivetrain: DrivetrainConfig::default(),
            stability: StabilityConfig::default(),
            aerodynamics: AerodynamicsConfig::default(),
        }
    }
}

impl GroundVehicle {
    /// Arcade preset — snappy acceleration, forgiving handling, high stability assists.
    ///
    /// Suitable for Mario Kart-style games, Rocket League, or casual driving.
    /// High torque, fast steering, generous grip, strong anti-roll and yaw damping.
    pub fn arcade_preset() -> Self {
        Self {
            mass_kg: 1_100.0,
            angular_inertia_kgm2: Vec3::new(600.0, 800.0, 900.0),
            center_of_mass_offset: Vec3::new(0.0, -0.45, 0.0),
            steering: SteeringConfig {
                max_angle_rad: 35.0_f32.to_radians(),
                steer_rate_rad_per_sec: 4.5,
                speed_reduction_start_mps: 15.0,
                speed_reduction_end_mps: 40.0,
                minimum_speed_factor: 0.40,
                ..SteeringConfig::default()
            },
            drivetrain: DrivetrainConfig {
                engine: EngineConfig {
                    peak_torque_nm: 600.0,
                    peak_torque_rpm: 4_500.0,
                    redline_rpm: 7_500.0,
                    idle_torque_fraction: 0.50,
                    redline_torque_fraction: 0.70,
                    engine_brake_torque_nm: 80.0,
                    ..EngineConfig::default()
                },
                transmission: TransmissionConfig {
                    final_drive_ratio: 3.50,
                    forward_gears: [3.20, 2.10, 1.50, 1.15, 0.92, 0.78],
                    forward_gear_count: 5,
                    shift_up_rpm: 6_500.0,
                    shift_down_rpm: 3_000.0,
                    clutch_coupling_speed_mps: 2.5,
                    ..TransmissionConfig::default()
                },
                brake_force_newtons: 18_000.0,
                handbrake_force_newtons: 14_000.0,
                drivetrain_efficiency: 0.92,
                ..DrivetrainConfig::default()
            },
            stability: StabilityConfig {
                anti_roll_force_n_per_ratio: 12_000.0,
                park_hold_force_newtons: 6_000.0,
                low_speed_traction_boost: 1.50,
                low_speed_traction_speed_threshold_mps: 4.0,
                yaw_stability_torque_nm_per_radps: 2_800.0,
                yaw_stability_speed_threshold_mps: 5.0,
                airborne_upright_torque_nm_per_rad: 2_000.0,
                ..StabilityConfig::default()
            },
            aerodynamics: AerodynamicsConfig {
                drag_force_per_speed_sq: 0.65,
                downforce_per_speed_sq: 0.30,
            },
        }
    }

    /// Simulation preset — realistic torque curve, nuanced grip, minimal assists.
    ///
    /// Suitable for Forza/Gran Turismo-style games or racing sims.
    /// Realistic mass, moderate torque, subtle stability aids, load-sensitive tires.
    pub fn simulation_preset() -> Self {
        Self {
            mass_kg: 1_450.0,
            angular_inertia_kgm2: Vec3::new(900.0, 1_100.0, 1_400.0),
            center_of_mass_offset: Vec3::new(0.0, -0.32, 0.0),
            steering: SteeringConfig {
                max_angle_rad: 28.0_f32.to_radians(),
                steer_rate_rad_per_sec: 2.4,
                speed_reduction_start_mps: 10.0,
                speed_reduction_end_mps: 28.0,
                minimum_speed_factor: 0.30,
                ..SteeringConfig::default()
            },
            drivetrain: DrivetrainConfig {
                engine: EngineConfig {
                    peak_torque_nm: 380.0,
                    peak_torque_rpm: 4_000.0,
                    redline_rpm: 6_500.0,
                    idle_torque_fraction: 0.40,
                    redline_torque_fraction: 0.58,
                    engine_brake_torque_nm: 120.0,
                    ..EngineConfig::default()
                },
                transmission: TransmissionConfig {
                    final_drive_ratio: 3.90,
                    forward_gears: [3.50, 2.30, 1.65, 1.25, 1.00, 0.85],
                    forward_gear_count: 6,
                    shift_up_rpm: 6_000.0,
                    shift_down_rpm: 2_800.0,
                    clutch_coupling_speed_mps: 4.5,
                    ..TransmissionConfig::default()
                },
                brake_force_newtons: 14_000.0,
                handbrake_force_newtons: 10_000.0,
                drivetrain_efficiency: 0.87,
                ..DrivetrainConfig::default()
            },
            stability: StabilityConfig {
                anti_roll_force_n_per_ratio: 5_000.0,
                park_hold_force_newtons: 4_000.0,
                low_speed_traction_boost: 1.10,
                low_speed_traction_speed_threshold_mps: 2.5,
                yaw_stability_torque_nm_per_radps: 800.0,
                yaw_stability_speed_threshold_mps: 10.0,
                airborne_upright_torque_nm_per_rad: 400.0,
                ..StabilityConfig::default()
            },
            aerodynamics: AerodynamicsConfig {
                drag_force_per_speed_sq: 1.10,
                downforce_per_speed_sq: 0.15,
            },
        }
    }

    /// Off-road preset — long-travel suspension, high traction, strong hill-hold.
    ///
    /// Suitable for rally, off-road, or adventure driving.
    /// Low gearing, high torque at low RPM, strong anti-roll, generous traction assists.
    pub fn offroad_preset() -> Self {
        Self {
            mass_kg: 1_600.0,
            angular_inertia_kgm2: Vec3::new(1_000.0, 1_200.0, 1_500.0),
            center_of_mass_offset: Vec3::new(0.0, -0.50, 0.0),
            steering: SteeringConfig {
                max_angle_rad: 26.0_f32.to_radians(),
                steer_rate_rad_per_sec: 2.2,
                speed_reduction_start_mps: 8.0,
                speed_reduction_end_mps: 18.0,
                minimum_speed_factor: 0.55,
                ..SteeringConfig::default()
            },
            drivetrain: DrivetrainConfig {
                engine: EngineConfig {
                    peak_torque_nm: 520.0,
                    peak_torque_rpm: 2_800.0,
                    redline_rpm: 5_000.0,
                    idle_torque_fraction: 0.65,
                    redline_torque_fraction: 0.60,
                    engine_brake_torque_nm: 90.0,
                    ..EngineConfig::default()
                },
                transmission: TransmissionConfig {
                    final_drive_ratio: 5.50,
                    forward_gears: [4.20, 2.60, 1.75, 1.30, 1.00, 0.82],
                    forward_gear_count: 5,
                    shift_up_rpm: 4_200.0,
                    shift_down_rpm: 2_000.0,
                    clutch_coupling_speed_mps: 1.5,
                    ..TransmissionConfig::default()
                },
                brake_force_newtons: 12_000.0,
                handbrake_force_newtons: 9_000.0,
                differential: DifferentialConfig {
                    mode: crate::DifferentialMode::LimitedSlip,
                    limited_slip_load_bias: 0.65,
                },
                drivetrain_efficiency: 0.88,
                ..DrivetrainConfig::default()
            },
            stability: StabilityConfig {
                anti_roll_force_n_per_ratio: 10_000.0,
                park_hold_force_newtons: 14_000.0,
                park_hold_speed_threshold_mps: 1.8,
                low_speed_traction_boost: 1.60,
                low_speed_traction_speed_threshold_mps: 3.5,
                yaw_stability_torque_nm_per_radps: 1_800.0,
                yaw_stability_speed_threshold_mps: 6.0,
                airborne_upright_torque_nm_per_rad: 1_500.0,
                ..StabilityConfig::default()
            },
            aerodynamics: AerodynamicsConfig {
                drag_force_per_speed_sq: 0.75,
                downforce_per_speed_sq: 0.05,
            },
        }
    }
}

/// Marker component — insert on a chassis entity to reset its drivetrain
/// (gear, engine RPM) and all associated wheel states (spin speed, slip, etc.)
/// back to defaults.  The plugin removes the marker after processing.
#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component, Debug)]
pub struct GroundVehicleReset;

#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component, Debug)]
pub struct GroundVehicleControl {
    pub throttle: f32,
    pub brake: f32,
    pub steering: f32,
    pub handbrake: f32,
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component, Debug)]
#[require(GroundVehicleWheelInternal, GroundVehicleWheelState)]
pub struct GroundVehicleWheel {
    pub chassis: Entity,
    pub axle: u8,
    pub side: WheelSide,
    pub drive_side: WheelSide,
    pub mount_point: Vec3,
    pub radius_m: f32,
    pub width_m: f32,
    pub rotational_inertia_kgm2: f32,
    pub steer_factor: f32,
    pub drive_factor: f32,
    pub brake_factor: f32,
    pub handbrake_factor: f32,
    pub suspension: SuspensionConfig,
    pub tire: TireGripConfig,
}

impl GroundVehicleWheel {
    pub fn default_front(chassis: Entity, mount_point: Vec3, side: WheelSide) -> Self {
        Self {
            chassis,
            axle: 0,
            side,
            drive_side: side,
            mount_point,
            radius_m: 0.36,
            width_m: 0.24,
            rotational_inertia_kgm2: 1.05,
            steer_factor: 1.0,
            drive_factor: 0.0,
            brake_factor: 1.0,
            handbrake_factor: 0.0,
            suspension: SuspensionConfig::default(),
            tire: TireGripConfig::default(),
        }
    }

    pub fn default_rear(chassis: Entity, mount_point: Vec3, side: WheelSide) -> Self {
        Self {
            chassis,
            axle: 1,
            side,
            drive_side: side,
            mount_point,
            radius_m: 0.36,
            width_m: 0.26,
            rotational_inertia_kgm2: 1.12,
            steer_factor: 0.0,
            drive_factor: 1.0,
            brake_factor: 1.0,
            handbrake_factor: 1.0,
            suspension: SuspensionConfig::default(),
            tire: TireGripConfig::default(),
        }
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component, Debug)]
pub struct GroundVehicleWheelVisual {
    pub visual_entity: Entity,
    pub visual_offset_local: Vec3,
    pub base_rotation: Quat,
    pub steering_axis_local: Vec3,
    pub rolling_axis_local: Vec3,
}

impl Default for GroundVehicleWheelVisual {
    fn default() -> Self {
        Self {
            visual_entity: Entity::PLACEHOLDER,
            visual_offset_local: Vec3::ZERO,
            base_rotation: Quat::IDENTITY,
            steering_axis_local: Vec3::Y,
            rolling_axis_local: Vec3::X,
        }
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component, Debug)]
pub struct GroundVehicleWheelState {
    pub grounded: bool,
    pub contact_entity: Option<Entity>,
    pub contact_point: Vec3,
    pub contact_normal: Vec3,
    pub suspension_length_m: f32,
    pub suspension_compression_m: f32,
    pub suspension_velocity_mps: f32,
    pub suspension_force_newtons: f32,
    pub load_newtons: f32,
    pub longitudinal_speed_mps: f32,
    pub lateral_speed_mps: f32,
    pub longitudinal_force_newtons: f32,
    pub lateral_force_newtons: f32,
    pub slip_ratio: f32,
    pub slip_angle_rad: f32,
    pub steer_angle_rad: f32,
    pub spin_angle_rad: f32,
    pub spin_speed_rad_per_sec: f32,
}

#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component, Debug)]
pub struct GroundVehicleTelemetry {
    pub speed_mps: f32,
    pub forward_speed_mps: f32,
    pub lateral_speed_mps: f32,
    pub grounded_wheels: u8,
    pub airborne: bool,
    pub average_steer_angle_rad: f32,
    pub drift_ratio: f32,
    pub drifting: bool,
    pub average_ground_normal: Vec3,
    pub engine_rpm: f32,
    pub selected_gear: i8,
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component, Debug)]
pub struct GroundVehicleSurface {
    pub longitudinal_grip_scale: f32,
    pub lateral_grip_scale: f32,
    pub rolling_drag_scale: f32,
    pub brake_scale: f32,
}

impl Default for GroundVehicleSurface {
    fn default() -> Self {
        Self {
            longitudinal_grip_scale: 1.0,
            lateral_grip_scale: 1.0,
            rolling_drag_scale: 1.0,
            brake_scale: 1.0,
        }
    }
}

#[derive(Resource, Reflect, Debug, Clone, Copy)]
#[reflect(Resource, Debug)]
pub struct GroundVehicleDebugDraw {
    pub enabled: bool,
    pub draw_suspension: bool,
    pub draw_contact_normals: bool,
    pub draw_force_vectors: bool,
    pub draw_slip_vectors: bool,
}

impl Default for GroundVehicleDebugDraw {
    fn default() -> Self {
        Self {
            enabled: false,
            draw_suspension: true,
            draw_contact_normals: true,
            draw_force_vectors: true,
            draw_slip_vectors: true,
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct GroundVehicleResolvedControl {
    pub throttle: f32,
    pub brake: f32,
    pub steering: f32,
    pub handbrake: f32,
    pub skid_left: f32,
    pub skid_right: f32,
    pub steer_speed_factor: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AxleAccumulator {
    pub left_count: u8,
    pub right_count: u8,
    pub left_compression_sum: f32,
    pub right_compression_sum: f32,
    pub left_contact_sum: Vec3,
    pub right_contact_sum: Vec3,
}

#[derive(Component, Debug, Clone)]
pub(crate) struct GroundVehicleInternal {
    pub was_airborne: bool,
    pub was_drifting: bool,
    pub engine_rpm: f32,
    pub selected_gear: i8,
    pub grounded_wheels: u8,
    pub average_ground_normal_sum: Vec3,
    pub drive_factor_sum: f32,
    pub drive_load_sum: f32,
    pub left_drive_factor_sum: f32,
    pub right_drive_factor_sum: f32,
    pub left_drive_load_sum: f32,
    pub right_drive_load_sum: f32,
    pub axle_accumulators: [AxleAccumulator; 8],
}

impl Default for GroundVehicleInternal {
    fn default() -> Self {
        Self {
            was_airborne: false,
            was_drifting: false,
            engine_rpm: 900.0,
            selected_gear: 1,
            grounded_wheels: 0,
            average_ground_normal_sum: Vec3::ZERO,
            drive_factor_sum: 0.0,
            drive_load_sum: 0.0,
            left_drive_factor_sum: 0.0,
            right_drive_factor_sum: 0.0,
            left_drive_load_sum: 0.0,
            right_drive_load_sum: 0.0,
            axle_accumulators: [AxleAccumulator::default(); 8],
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct GroundVehicleWheelInternal {
    pub previous_grounded: bool,
    pub previous_contact_entity: Option<Entity>,
    pub previous_suspension_length_m: f32,
    pub drive_force_request_newtons: f32,
    pub brake_force_request_newtons: f32,
}
