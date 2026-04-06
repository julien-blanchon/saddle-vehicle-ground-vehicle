use bevy::prelude::*;

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SteeringMode {
    Road,
    Disabled,
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

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct DifferentialConfig {
    pub mode: DifferentialMode,
    pub limited_slip_load_bias: f32,
}

impl Default for DifferentialConfig {
    fn default() -> Self {
        Self {
            mode: DifferentialMode::LimitedSlip,
            limited_slip_load_bias: 0.55,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DirectionChangePolicy {
    Immediate,
    StopThenChange,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct DirectionChangeConfig {
    pub policy: DirectionChangePolicy,
    pub speed_threshold_mps: f32,
}

impl Default for DirectionChangeConfig {
    fn default() -> Self {
        Self {
            policy: DirectionChangePolicy::StopThenChange,
            speed_threshold_mps: 1.25,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct EngineConfig {
    pub idle_rpm: f32,
    pub peak_torque_nm: f32,
    pub peak_torque_rpm: f32,
    pub redline_rpm: f32,
    pub idle_torque_fraction: f32,
    pub redline_torque_fraction: f32,
    pub engine_brake_torque_nm: f32,
}

impl EngineConfig {
    pub fn torque_at_rpm(self, rpm: f32) -> f32 {
        let idle_rpm = self.idle_rpm.max(100.0);
        let redline_rpm = self.redline_rpm.max(idle_rpm + 100.0);
        let peak_rpm = self.peak_torque_rpm.clamp(idle_rpm, redline_rpm);
        let clamped_rpm = rpm.clamp(idle_rpm, redline_rpm);
        let peak_torque = self.peak_torque_nm.max(0.0);
        let idle_torque = peak_torque * self.idle_torque_fraction.max(0.0);
        let redline_torque = peak_torque * self.redline_torque_fraction.max(0.0);

        if clamped_rpm <= peak_rpm {
            let t = (clamped_rpm - idle_rpm) / (peak_rpm - idle_rpm).max(1.0);
            idle_torque.lerp(peak_torque, t)
        } else {
            let t = (clamped_rpm - peak_rpm) / (redline_rpm - peak_rpm).max(1.0);
            peak_torque.lerp(redline_torque, t)
        }
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            idle_rpm: 900.0,
            peak_torque_nm: 480.0,
            peak_torque_rpm: 4_200.0,
            redline_rpm: 6_800.0,
            idle_torque_fraction: 0.45,
            redline_torque_fraction: 0.62,
            engine_brake_torque_nm: 100.0,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct AutomaticGearboxConfig {
    pub forward_gears: [f32; 6],
    pub forward_gear_count: u8,
    pub final_drive_ratio: f32,
    pub reverse_ratio: f32,
    pub shift_up_rpm: f32,
    pub shift_down_rpm: f32,
    pub coupling_speed_mps: f32,
    pub direction_change: DirectionChangeConfig,
}

impl AutomaticGearboxConfig {
    pub fn gear_ratio(self, gear: i8) -> f32 {
        match gear.cmp(&0) {
            std::cmp::Ordering::Less => self.reverse_ratio.abs() * self.final_drive_ratio.abs(),
            std::cmp::Ordering::Equal => 0.0,
            std::cmp::Ordering::Greater => {
                let max_index = usize::from(self.forward_gear_count.saturating_sub(1))
                    .min(self.forward_gears.len().saturating_sub(1));
                let index = usize::try_from((gear - 1).max(0))
                    .unwrap_or(0)
                    .min(max_index);
                self.forward_gears[index].abs() * self.final_drive_ratio.abs()
            }
        }
    }

    pub fn max_forward_gear(self) -> i8 {
        self.forward_gear_count
            .clamp(1, self.forward_gears.len() as u8) as i8
    }
}

impl Default for AutomaticGearboxConfig {
    fn default() -> Self {
        Self {
            forward_gears: [3.45, 2.25, 1.62, 1.22, 0.98, 0.84],
            forward_gear_count: 5,
            final_drive_ratio: 3.85,
            reverse_ratio: 3.10,
            shift_up_rpm: 5_900.0,
            shift_down_rpm: 2_600.0,
            coupling_speed_mps: 4.0,
            direction_change: DirectionChangeConfig::default(),
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct FixedGearConfig {
    pub forward_ratio: f32,
    pub reverse_ratio: f32,
    pub coupling_speed_mps: f32,
    pub direction_change: DirectionChangeConfig,
}

impl FixedGearConfig {
    pub fn gear_ratio(self, gear: i8) -> f32 {
        match gear.cmp(&0) {
            std::cmp::Ordering::Less => self.reverse_ratio.abs(),
            std::cmp::Ordering::Equal => 0.0,
            std::cmp::Ordering::Greater => self.forward_ratio.abs(),
        }
    }
}

impl Default for FixedGearConfig {
    fn default() -> Self {
        Self {
            forward_ratio: 3.85,
            reverse_ratio: 3.10,
            coupling_speed_mps: 4.0,
            direction_change: DirectionChangeConfig::default(),
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub enum GearModel {
    Automatic(AutomaticGearboxConfig),
    Fixed(FixedGearConfig),
}

impl Default for GearModel {
    fn default() -> Self {
        Self::Automatic(AutomaticGearboxConfig::default())
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct AxleDriveConfig {
    pub differential: DifferentialConfig,
    pub drivetrain_efficiency: f32,
}

impl Default for AxleDriveConfig {
    fn default() -> Self {
        Self {
            differential: DifferentialConfig::default(),
            drivetrain_efficiency: 0.90,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct TrackDriveConfig {
    pub differential: DifferentialConfig,
    pub drivetrain_efficiency: f32,
    pub turn_split: f32,
}

impl Default for TrackDriveConfig {
    fn default() -> Self {
        Self {
            differential: DifferentialConfig::default(),
            drivetrain_efficiency: 0.90,
            turn_split: 0.85,
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub enum DriveModel {
    Axle(AxleDriveConfig),
    Track(TrackDriveConfig),
}

impl Default for DriveModel {
    fn default() -> Self {
        Self::Axle(AxleDriveConfig::default())
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct PowertrainConfig {
    pub engine: EngineConfig,
    pub drive_model: DriveModel,
    pub gear_model: GearModel,
    pub brake_force_newtons: f32,
    pub auxiliary_brake_force_newtons: f32,
}

impl Default for PowertrainConfig {
    fn default() -> Self {
        Self {
            engine: EngineConfig::default(),
            drive_model: DriveModel::default(),
            gear_model: GearModel::default(),
            brake_force_newtons: 12_000.0,
            auxiliary_brake_force_newtons: 10_500.0,
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

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TireModel {
    Linear,
    MagicFormula,
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct MagicFormulaConfig {
    pub longitudinal_b: f32,
    pub longitudinal_c: f32,
    pub longitudinal_e: f32,
    pub longitudinal_peak_slip_ratio: f32,
    pub lateral_b: f32,
    pub lateral_c: f32,
    pub lateral_e: f32,
    pub lateral_peak_slip_angle_rad: f32,
}

impl Default for MagicFormulaConfig {
    fn default() -> Self {
        Self {
            longitudinal_b: 10.5,
            longitudinal_c: 1.72,
            longitudinal_e: 0.32,
            longitudinal_peak_slip_ratio: 0.12,
            lateral_b: 7.8,
            lateral_c: 1.38,
            lateral_e: 0.24,
            lateral_peak_slip_angle_rad: 10.0_f32.to_radians(),
        }
    }
}

#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct TireGripConfig {
    pub model: TireModel,
    pub longitudinal_grip: f32,
    pub lateral_grip: f32,
    pub longitudinal_stiffness: f32,
    pub lateral_stiffness: f32,
    pub lateral_response_exponent: f32,
    pub rolling_resistance_force_newtons: f32,
    pub auxiliary_brake_lateral_multiplier: f32,
    pub auxiliary_brake_longitudinal_multiplier: f32,
    pub low_speed_lateral_multiplier: f32,
    pub nominal_load_newtons: f32,
    pub load_sensitivity: f32,
    pub low_speed_slip_reference_mps: f32,
    pub magic_formula: MagicFormulaConfig,
}

impl Default for TireGripConfig {
    fn default() -> Self {
        Self {
            model: TireModel::Linear,
            longitudinal_grip: 1.35,
            lateral_grip: 1.15,
            longitudinal_stiffness: 170.0,
            lateral_stiffness: 460.0,
            lateral_response_exponent: 1.0,
            rolling_resistance_force_newtons: 32.0,
            auxiliary_brake_lateral_multiplier: 0.42,
            auxiliary_brake_longitudinal_multiplier: 0.20,
            low_speed_lateral_multiplier: 1.35,
            nominal_load_newtons: 3_500.0,
            load_sensitivity: 0.45,
            low_speed_slip_reference_mps: 2.5,
            magic_formula: MagicFormulaConfig::default(),
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
}

impl Default for StabilityConfig {
    fn default() -> Self {
        Self {
            anti_roll_force_n_per_ratio: 8_000.0,
            park_hold_force_newtons: 5_500.0,
            park_hold_speed_threshold_mps: 0.65,
            low_speed_traction_boost: 1.30,
            low_speed_traction_speed_threshold_mps: 3.5,
            yaw_stability_torque_nm_per_radps: 2_000.0,
            yaw_stability_speed_threshold_mps: 6.0,
            airborne_upright_torque_nm_per_rad: 1_200.0,
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
            drag_force_per_speed_sq: 0.85,
            downforce_per_speed_sq: 0.22,
        }
    }
}
