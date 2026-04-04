use crate::config::{
    AerodynamicsConfig, DrivetrainConfig, StabilityConfig, SteeringConfig, SuspensionConfig,
    TireGripConfig,
};
use avian3d::prelude::*;
use bevy::prelude::*;

const DEFAULT_MASS_KG: f32 = 1_300.0;
const DEFAULT_COM_OFFSET: Vec3 = Vec3::new(0.0, -0.35, 0.0);
const DEFAULT_INERTIA: Vec3 = Vec3::new(650.0, 820.0, 1_050.0);

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
