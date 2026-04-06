# Configuration

Defaults below refer to each type's `Default` implementation unless noted otherwise.

## Conventions

- Distances are meters.
- Speeds are meters per second.
- Forces are newtons.
- Torques are newton-meters.
- Angles are radians.
- `VehicleIntent` inputs are normalized: `drive` and `turn` in `[-1, 1]`, `brake` and `auxiliary_brake` in `[0, 1]`.

## `GroundVehicle`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `mass_kg` | `f32` | `1300.0` | Chassis mass used by Avian |
| `angular_inertia_kgm2` | `Vec3` | `(800, 1000, 1200)` | Rotational inertia about local X/Y/Z |
| `center_of_mass_offset` | `Vec3` | `(0.0, -0.40, 0.0)` | Offsets the rigid-body center of mass |
| `steering` | `SteeringConfig` | default | Wheel-angle response |
| `powertrain` | `PowertrainConfig` | default | Torque delivery, ratio selection, and braking policy |
| `stability` | `StabilityConfig` | default | Assists such as anti-roll, hill hold, yaw damping, upright torque |
| `aerodynamics` | `AerodynamicsConfig` | default | Drag and downforce |

## `VehicleIntent`

| Field | Type | Range | Effect |
| --- | --- | --- | --- |
| `drive` | `f32` | `[-1, 1]` | Signed propulsion demand |
| `turn` | `f32` | `[-1, 1]` | Signed turn demand |
| `brake` | `f32` | `[0, 1]` | Primary brake demand |
| `auxiliary_brake` | `f32` | `[0, 1]` | Secondary brake channel used by auxiliary brake budgets and per-wheel factors |

## `GroundVehicleWheel`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `chassis` | `Entity` | required | Chassis rigid body that owns the wheel |
| `axle` | `u8` | helper-specific | Groups wheels for anti-roll |
| `side` | `WheelSide` | helper-specific | Left-right pairing and Ackermann side |
| `drive_side` | `WheelSide` | helper-specific | Left-right drive-group assignment for `DriveModel::Track` |
| `mount_point` | `Vec3` | helper-specific | Suspension origin in chassis-local space |
| `radius_m` | `f32` | `0.36` | Wheel radius for casts and spin |
| `width_m` | `f32` | helper-specific | Authoring/debug width; useful for visuals |
| `rotational_inertia_kgm2` | `f32` | helper-specific | Wheel angular inertia used by slip-ratio and torque response |
| `steer_factor` | `f32` | helper-specific | How much of the steering angle this wheel receives |
| `drive_factor` | `f32` | helper-specific | Torque-share weight |
| `brake_factor` | `f32` | `1.0` | Primary-brake share |
| `auxiliary_brake_factor` | `f32` | helper-specific | Secondary-brake share |
| `suspension` | `SuspensionConfig` | default | Suspension travel and support force |
| `tire` | `TireGripConfig` | default | Per-wheel tire response |

## `SteeringConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `mode` | `SteeringMode` | `Road` | `Road` resolves wheel angles, `Disabled` keeps steering at zero |
| `max_angle_rad` | `f32` | `0.5585` | Maximum steer lock before Ackermann blend |
| `steer_rate_rad_per_sec` | `f32` | `2.8` | Slew rate toward target steer angle |
| `ackermann_ratio` | `f32` | `0.85` | Blend from parallel steer to Ackermann |
| `speed_reduction_start_mps` | `f32` | `12.0` | Speed where steering reduction begins |
| `speed_reduction_end_mps` | `f32` | `32.0` | Speed where minimum steering factor is reached |
| `minimum_speed_factor` | `f32` | `0.35` | Minimum steering scale at speed |
| `wheelbase_override_m` | `Option<f32>` | `None` | Explicit Ackermann wheelbase override |
| `track_width_override_m` | `Option<f32>` | `None` | Explicit Ackermann track-width override |

## `PowertrainConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `engine` | `EngineConfig` | default | Torque curve and engine-braking behavior |
| `drive_model` | `DriveModel` | `DriveModel::Axle` | Chooses torque-distribution strategy |
| `gear_model` | `GearModel` | `GearModel::Automatic` | Chooses ratio-selection strategy |
| `brake_force_newtons` | `f32` | `12000.0` | Primary brake force budget |
| `auxiliary_brake_force_newtons` | `f32` | `10500.0` | Secondary brake force budget |

### `DriveModel`

- `DriveModel::Axle(AxleDriveConfig)`:
  Uses authored `drive_factor` weights across all driven wheels.
- `DriveModel::Track(TrackDriveConfig)`:
  Uses authored `drive_side` plus signed turn splitting for left/right track-drive behavior.

### `AxleDriveConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `differential` | `DifferentialConfig` | default | Torque split behavior across driven wheels |
| `drivetrain_efficiency` | `f32` | `0.90` | Multiplies delivered wheel torque after gearing |

### `TrackDriveConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `differential` | `DifferentialConfig` | default | Torque split behavior within each side |
| `drivetrain_efficiency` | `f32` | `0.90` | Multiplies delivered wheel torque after gearing |
| `turn_split` | `f32` | `0.85` | Converts signed `turn` intent into left/right drive bias |

### `GearModel`

- `GearModel::Automatic(AutomaticGearboxConfig)`:
  Multi-ratio automatic forward gears plus reverse handling.
- `GearModel::Fixed(FixedGearConfig)`:
  Single forward/reverse ratio with explicit direction-change behavior.

### `AutomaticGearboxConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `forward_gears` | `[f32; 6]` | `[3.45, 2.25, 1.62, 1.22, 0.98, 0.84]` | Forward ratios before final drive |
| `forward_gear_count` | `u8` | `5` | Active number of usable forward gears |
| `final_drive_ratio` | `f32` | `3.85` | Global multiplier applied to all forward and reverse gears |
| `reverse_ratio` | `f32` | `3.10` | Reverse ratio before final drive |
| `shift_up_rpm` | `f32` | `5900.0` | Automatic upshift threshold |
| `shift_down_rpm` | `f32` | `2600.0` | Automatic downshift threshold |
| `coupling_speed_mps` | `f32` | `4.0` | How quickly wheel speed dominates the free-rev target |
| `direction_change` | `DirectionChangeConfig` | default | How signed drive input transitions between forward and reverse |

### `FixedGearConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `forward_ratio` | `f32` | `3.85` | Single forward ratio |
| `reverse_ratio` | `f32` | `3.10` | Single reverse ratio |
| `coupling_speed_mps` | `f32` | `4.0` | How quickly wheel speed dominates the free-rev target |
| `direction_change` | `DirectionChangeConfig` | default | How signed drive input transitions between forward and reverse |

### `DirectionChangeConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `policy` | `DirectionChangePolicy` | `StopThenChange` | `Immediate` flips direction instantly, `StopThenChange` converts opposite signed drive into braking until speed falls below threshold |
| `speed_threshold_mps` | `f32` | `1.25` | Threshold used by `StopThenChange` |

### `DifferentialConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `mode` | `DifferentialMode` | `LimitedSlip` | `Open`, `LimitedSlip`, or `Spool` |
| `limited_slip_load_bias` | `f32` | `0.55` | Blend between authored drive-factor share and load-based share when in `LimitedSlip` mode |

## `EngineConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `idle_rpm` | `f32` | `900.0` | Lower clamp for the engine speed estimate |
| `peak_torque_nm` | `f32` | `480.0` | Peak torque value used by the authored curve |
| `peak_torque_rpm` | `f32` | `4200.0` | Where the torque curve reaches peak output |
| `redline_rpm` | `f32` | `6800.0` | Upper clamp for engine speed and torque falloff |
| `idle_torque_fraction` | `f32` | `0.45` | Fraction of peak torque available near idle |
| `redline_torque_fraction` | `f32` | `0.62` | Fraction of peak torque remaining at redline |
| `engine_brake_torque_nm` | `f32` | `100.0` | Passive driveline drag when off drive input |

## `SuspensionConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `rest_length_m` | `f32` | `0.38` | Target suspension length around neutral ride height |
| `max_compression_m` | `f32` | `0.18` | Allowed travel into bump |
| `max_droop_m` | `f32` | `0.16` | Allowed extension beyond rest |
| `spring_strength_n_per_m` | `f32` | `29000.0` | Spring force from compression |
| `damper_strength_n_per_mps` | `f32` | `3600.0` | Damping from suspension velocity |
| `bump_stop_strength_n_per_m` | `f32` | `18000.0` | Extra force once past minimum length |

## `TireGripConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `model` | `TireModel` | `Linear` | Selects the tire-force response model |
| `longitudinal_grip` | `f32` | `1.35` | Longitudinal grip limit relative to wheel load |
| `lateral_grip` | `f32` | `1.15` | Lateral grip limit relative to wheel load |
| `longitudinal_stiffness` | `f32` | `170.0` | Passive correction along wheel forward |
| `lateral_stiffness` | `f32` | `460.0` | Passive correction across the tire |
| `lateral_response_exponent` | `f32` | `1.0` | Shapes linear lateral response |
| `rolling_resistance_force_newtons` | `f32` | `32.0` | Passive rolling drag |
| `auxiliary_brake_lateral_multiplier` | `f32` | `0.42` | Lateral grip scaling under auxiliary brake |
| `auxiliary_brake_longitudinal_multiplier` | `f32` | `0.20` | Longitudinal grip scaling under auxiliary brake |
| `low_speed_lateral_multiplier` | `f32` | `1.35` | Extra lateral grip near stop |
| `nominal_load_newtons` | `f32` | `3500.0` | Reference load for sensitivity scaling |
| `load_sensitivity` | `f32` | `0.45` | How strongly grip changes with load |
| `low_speed_slip_reference_mps` | `f32` | `2.5` | Slip-speed floor used by the low-speed slip-ratio estimate |
| `magic_formula` | `MagicFormulaConfig` | default | Shape parameters used when `model = MagicFormula` |

## `MagicFormulaConfig`

| Field | Type | Default |
| --- | --- | --- |
| `longitudinal_b` | `f32` | `10.5` |
| `longitudinal_c` | `f32` | `1.72` |
| `longitudinal_e` | `f32` | `0.32` |
| `longitudinal_peak_slip_ratio` | `f32` | `0.12` |
| `lateral_b` | `f32` | `7.8` |
| `lateral_c` | `f32` | `1.38` |
| `lateral_e` | `f32` | `0.24` |
| `lateral_peak_slip_angle_rad` | `f32` | `10°` |

## `StabilityConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `anti_roll_force_n_per_ratio` | `f32` | `8000.0` | Anti-roll support from left-right compression delta |
| `park_hold_force_newtons` | `f32` | `5500.0` | Tangent force budget for hill hold |
| `park_hold_speed_threshold_mps` | `f32` | `0.65` | Speed below which hill hold can engage |
| `low_speed_traction_boost` | `f32` | `1.30` | Additional longitudinal support near stop |
| `low_speed_traction_speed_threshold_mps` | `f32` | `3.5` | Window where the boost is active |
| `yaw_stability_torque_nm_per_radps` | `f32` | `2000.0` | High-speed yaw damping assist |
| `yaw_stability_speed_threshold_mps` | `f32` | `6.0` | Minimum speed for yaw damping |
| `airborne_upright_torque_nm_per_rad` | `f32` | `1200.0` | Upright assist while airborne |

## `AerodynamicsConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `drag_force_per_speed_sq` | `f32` | `0.85` | Speed-squared drag |
| `downforce_per_speed_sq` | `f32` | `0.22` | Speed-squared downforce along chassis down |

## Optional Drift Helper

Drift state is now provided by the optional helper layer instead of the core telemetry component.

### `GroundVehicleDriftConfig`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `entry_ratio` | `f32` | `0.34` | Drift threshold when not already drifting |
| `exit_ratio` | `f32` | `0.24` | Exit threshold with hysteresis |
| `minimum_forward_speed_mps` | `f32` | `5.0` | Minimum forward speed before drift state can become active |

### `GroundVehicleDriftTelemetry`

| Field | Type | Effect |
| --- | --- | --- |
| `drift_ratio` | `f32` | Slip-based aggregate drift ratio |
| `drifting` | `bool` | Hysteresis-smoothed drift state |

Attach `GroundVehicleDriftConfig` and add `GroundVehicleDriftPlugin` only when your game actually wants drift telemetry or drift messages.
