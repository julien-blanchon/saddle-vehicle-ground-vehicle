use bevy::prelude::*;

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SteeringMode {
    Road,
    SkidSteer,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct SteeringConfig {
    pub mode: SteeringMode,
    pub max_angle_rad: f32,
    pub steer_rate_rad_per_sec: f32,
    pub ackermann_ratio: f32,
    pub speed_reduction_start_mps: f32,
    pub speed_reduction_end_mps: f32,
    pub minimum_speed_factor: f32,
    pub skid_steer_turn_scale: f32,
    pub wheelbase_override_m: Option<f32>,
    pub track_width_override_m: Option<f32>,
}

impl Default for SteeringConfig {
    fn default() -> Self {
        Self {
            mode: SteeringMode::Road,
            max_angle_rad: 32.0_f32.to_radians(),
            steer_rate_rad_per_sec: 2.8,
            ackermann_ratio: 0.85,
            speed_reduction_start_mps: 12.0,
            speed_reduction_end_mps: 32.0,
            minimum_speed_factor: 0.35,
            skid_steer_turn_scale: 0.85,
            wheelbase_override_m: None,
            track_width_override_m: None,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DifferentialMode {
    Open,
    LimitedSlip,
    Spool,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReversePolicy {
    Immediate,
    StopThenReverse,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct DrivetrainConfig {
    pub differential: DifferentialMode,
    pub reverse_policy: ReversePolicy,
    pub max_drive_force_newtons: f32,
    pub max_reverse_force_newtons: f32,
    pub brake_force_newtons: f32,
    pub handbrake_force_newtons: f32,
    pub engine_brake_force_newtons: f32,
    pub reverse_speed_threshold_mps: f32,
    pub limited_slip_load_bias: f32,
}

impl Default for DrivetrainConfig {
    fn default() -> Self {
        Self {
            differential: DifferentialMode::LimitedSlip,
            reverse_policy: ReversePolicy::StopThenReverse,
            max_drive_force_newtons: 9_500.0,
            max_reverse_force_newtons: 5_800.0,
            brake_force_newtons: 12_000.0,
            handbrake_force_newtons: 10_500.0,
            engine_brake_force_newtons: 2_200.0,
            reverse_speed_threshold_mps: 1.25,
            limited_slip_load_bias: 0.55,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct SuspensionConfig {
    pub rest_length_m: f32,
    pub max_compression_m: f32,
    pub max_droop_m: f32,
    pub spring_strength_n_per_m: f32,
    pub damper_strength_n_per_mps: f32,
    pub bump_stop_strength_n_per_m: f32,
}

impl SuspensionConfig {
    pub fn min_length(self) -> f32 {
        (self.rest_length_m - self.max_compression_m).max(0.01)
    }

    pub fn max_length(self) -> f32 {
        self.rest_length_m + self.max_droop_m
    }

    pub fn total_travel(self) -> f32 {
        (self.max_length() - self.min_length()).max(0.001)
    }
}

impl Default for SuspensionConfig {
    fn default() -> Self {
        Self {
            rest_length_m: 0.38,
            max_compression_m: 0.18,
            max_droop_m: 0.16,
            spring_strength_n_per_m: 29_000.0,
            damper_strength_n_per_mps: 3_600.0,
            bump_stop_strength_n_per_m: 18_000.0,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct TireGripConfig {
    pub longitudinal_grip: f32,
    pub lateral_grip: f32,
    pub longitudinal_stiffness: f32,
    pub lateral_stiffness: f32,
    pub lateral_response_exponent: f32,
    pub rolling_resistance_force_newtons: f32,
    pub handbrake_lateral_multiplier: f32,
    pub handbrake_longitudinal_multiplier: f32,
    pub low_speed_lateral_multiplier: f32,
    pub nominal_load_newtons: f32,
    pub load_sensitivity: f32,
}

impl Default for TireGripConfig {
    fn default() -> Self {
        Self {
            longitudinal_grip: 1.35,
            lateral_grip: 1.15,
            longitudinal_stiffness: 170.0,
            lateral_stiffness: 460.0,
            lateral_response_exponent: 1.0,
            rolling_resistance_force_newtons: 32.0,
            handbrake_lateral_multiplier: 0.42,
            handbrake_longitudinal_multiplier: 0.20,
            low_speed_lateral_multiplier: 1.35,
            nominal_load_newtons: 3_500.0,
            load_sensitivity: 0.45,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct StabilityConfig {
    pub anti_roll_force_n_per_ratio: f32,
    pub park_hold_force_newtons: f32,
    pub park_hold_speed_threshold_mps: f32,
    pub low_speed_traction_boost: f32,
    pub low_speed_traction_speed_threshold_mps: f32,
    pub yaw_stability_torque_nm_per_radps: f32,
    pub yaw_stability_speed_threshold_mps: f32,
    pub airborne_upright_torque_nm_per_rad: f32,
    pub drift_entry_ratio: f32,
    pub drift_exit_ratio: f32,
}

impl Default for StabilityConfig {
    fn default() -> Self {
        Self {
            anti_roll_force_n_per_ratio: 3_000.0,
            park_hold_force_newtons: 4_500.0,
            park_hold_speed_threshold_mps: 0.65,
            low_speed_traction_boost: 1.25,
            low_speed_traction_speed_threshold_mps: 3.0,
            yaw_stability_torque_nm_per_radps: 1_500.0,
            yaw_stability_speed_threshold_mps: 8.0,
            airborne_upright_torque_nm_per_rad: 850.0,
            drift_entry_ratio: 0.34,
            drift_exit_ratio: 0.24,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct AerodynamicsConfig {
    pub drag_force_per_speed_sq: f32,
    pub downforce_per_speed_sq: f32,
}

impl Default for AerodynamicsConfig {
    fn default() -> Self {
        Self {
            drag_force_per_speed_sq: 1.05,
            downforce_per_speed_sq: 0.18,
        }
    }
}
