# Configuration

Defaults below refer to each type's `Default` implementation unless noted otherwise.

## Conventions

- Distances are meters.
- Speeds are meters per second.
- Forces are newtons.
- Torques are newton-meters.
- Angles are radians.
- `GroundVehicleControl` inputs are normalized: throttle and steering in `[-1, 1]`, brake and handbrake in `[0, 1]`.

## `GroundVehicle`

| Field | Type | Unit | Default | Practical range | Effect | Common failure when mis-tuned |
| --- | --- | --- | --- | --- | --- | --- |
| `mass_kg` | `f32` | kg | `1300.0` | `> 0` | Chassis mass used by Avian | Too low feels floaty; too high feels dead |
| `angular_inertia_kgm2` | `Vec3` | kg m^2 | `(650, 820, 1050)` | each axis `> 0` | Rotational inertia about local X/Y/Z | Too low snaps or rolls violently |
| `center_of_mass_offset` | `Vec3` | m | `(0.0, -0.35, 0.0)` | finite | Offsets the rigid-body center of mass | Too high causes rollover and curb instability |
| `steering` | `SteeringConfig` | n/a | default | see below | Steering mode and angle response | Wrong mode or weak angle cap makes turning ineffective |
| `drivetrain` | `DrivetrainConfig` | n/a | default | see below | Force delivery and brake policy | Too much force overwhelms grip and causes wheel-hop-like behavior |
| `stability` | `StabilityConfig` | n/a | default | see below | Assists, anti-roll, drift thresholds | Over-tuned assists make the vehicle feel glued and artificial |
| `aerodynamics` | `AerodynamicsConfig` | n/a | default | see below | Speed-squared drag and downforce | Too much downforce hides suspension and grip tuning issues |

## `GroundVehicleControl`

| Field | Type | Unit | Default | Range | Effect |
| --- | --- | --- | --- | --- | --- |
| `throttle` | `f32` | normalized | `0.0` | `[-1, 1]` | Positive for forward drive, negative for reverse intent |
| `brake` | `f32` | normalized | `0.0` | `[0, 1]` | Service brake demand |
| `steering` | `f32` | normalized | `0.0` | `[-1, 1]` | Steering input or turn demand |
| `handbrake` | `f32` | normalized | `0.0` | `[0, 1]` | Handbrake or rear-lock demand |

## `GroundVehicleWheel`

| Field | Type | Unit | Default | Practical range | Effect | Common failure when mis-tuned |
| --- | --- | --- | --- | --- | --- | --- |
| `chassis` | `Entity` | n/a | required | valid entity | Chassis rigid body that owns the wheel | Wrong entity silently disconnects the wheel |
| `axle` | `u8` | index | `0` or `1` in helpers | `0..=7` used internally | Groups wheels for anti-roll | Wrong axle index makes anti-roll act on the wrong pair |
| `side` | `WheelSide` | n/a | helper-specific | left/right/center | Left-right pairing and Ackermann side | Wrong side swaps steering behavior |
| `drive_side` | `WheelSide` | n/a | helper-specific | left/right/center | Left-right drive-group assignment | Wrong value breaks skid steer torque split |
| `mount_point` | `Vec3` | m | helper-specific | finite | Suspension origin in chassis-local space | Wrong height causes clipping or airborne wheels |
| `radius_m` | `f32` | m | `0.36` | `> 0` | Wheel radius for casts and spin | Too small sinks into terrain; too large hovers |
| `width_m` | `f32` | m | `0.24` or `0.26` | `> 0` | Authoring/debug width; useful for visuals | Visual mismatch if not kept close to the mesh |
| `steer_factor` | `f32` | scale | helper-specific | `0..=1` typical | How much of the steering angle this wheel receives | Unexpected four-wheel steer or dead front axle |
| `drive_factor` | `f32` | weight | helper-specific | `>= 0` | Torque-share weight | Torque missing from intended driven axle |
| `brake_factor` | `f32` | weight | `1.0` | `>= 0` | Service-brake share | Vehicle pulls under braking if side-to-side mismatch exists |
| `handbrake_factor` | `f32` | weight | helper-specific | `>= 0` | Handbrake share | Drift setup never rotates or locks the wrong axle |
| `suspension` | `SuspensionConfig` | n/a | default | see below | Suspension travel and support force | Wrong travel or spring rate causes bottoming or pogoing |
| `tire` | `TireGripConfig` | n/a | default | see below | Per-wheel tire response | Too much stiffness causes twitchy snap forces |

## `GroundVehicleWheelVisual`

| Field | Type | Unit | Default | Effect |
| --- | --- | --- | --- | --- |
| `visual_entity` | `Entity` | n/a | placeholder | The visible mesh or scene entity to drive |
| `visual_offset_local` | `Vec3` | m | `Vec3::ZERO` | Local offset after suspension placement |
| `base_rotation` | `Quat` | n/a | identity | Base wheel orientation, usually to rotate a cylinder onto the axle |
| `steering_axis_local` | `Vec3` | direction | `Vec3::Y` | Local axis used for steer angle |
| `rolling_axis_local` | `Vec3` | direction | `Vec3::X` | Local axis used for spin angle |

## `SteeringConfig`

| Field | Type | Unit | Default | Practical range | Effect | Common failure when mis-tuned |
| --- | --- | --- | --- | --- | --- | --- |
| `mode` | `SteeringMode` | n/a | `Road` | `Road` or `SkidSteer` | Chooses steering model | Wrong mode makes a tank steer like a car or vice versa |
| `max_angle_rad` | `f32` | rad | `0.5585` | `0.2..1.0` | Maximum steer lock before Ackermann blend | Too large causes twitchy high-speed yaw |
| `steer_rate_rad_per_sec` | `f32` | rad/s | `2.8` | `> 0` | Slew rate toward target steer angle | Too low feels laggy; too high feels digital |
| `ackermann_ratio` | `f32` | ratio | `0.85` | `0..=1` | Blend from parallel steer to Ackermann | Too high with wrong geometry over-rotates inside wheel |
| `speed_reduction_start_mps` | `f32` | m/s | `12.0` | `>= 0` | Speed where steering reduction begins | Too low makes parking-lot steering weak |
| `speed_reduction_end_mps` | `f32` | m/s | `32.0` | `> start` | Speed where min factor is reached | Too high leaves twitchy highway steering |
| `minimum_speed_factor` | `f32` | ratio | `0.35` | `0..=1` | Minimum steering scale at speed | Too low makes fast vehicles refuse to turn |
| `skid_steer_turn_scale` | `f32` | ratio | `0.85` | `>= 0` | Converts steering input into left-right drive split | Too high makes skid vehicles spin in place too abruptly |
| `wheelbase_override_m` | `Option<f32>` | m | `None` | `> 0` | Explicit Ackermann wheelbase. If `None`, the crate derives it from the wheel layout of the strongest steer axle and the remaining paired axles on the chassis. | Wrong value warps left/right steer asymmetry |
| `track_width_override_m` | `Option<f32>` | m | `None` | `> 0` | Explicit Ackermann track width. If `None`, the crate derives it from the left/right spacing of the strongest steer axle. | Wrong value exaggerates inside/outside mismatch |

## `DrivetrainConfig`

| Field | Type | Unit | Default | Practical range | Effect | Common failure when mis-tuned |
| --- | --- | --- | --- | --- | --- | --- |
| `differential` | `DifferentialMode` | n/a | `LimitedSlip` | open / limited-slip / spool | Chooses torque split behavior | Open diff can unload badly off-road; spool can bind on road |
| `reverse_policy` | `ReversePolicy` | n/a | `StopThenReverse` | immediate / stop-then-reverse | Resolves ambiguous reverse input | Wrong policy feels sticky or too arcade-like |
| `max_drive_force_newtons` | `f32` | N | `9500.0` | `>= 0` | Forward traction budget | Too high overwhelms tires and creates snap oversteer |
| `max_reverse_force_newtons` | `f32` | N | `5800.0` | `>= 0` | Reverse traction budget | Too low prevents hill climbs in reverse |
| `brake_force_newtons` | `f32` | N | `12000.0` | `>= 0` | Service-brake force budget | Too high causes instant lock and jitter |
| `handbrake_force_newtons` | `f32` | N | `10500.0` | `>= 0` | Handbrake force budget | Too low never initiates rotation; too high freezes the rear axle |
| `engine_brake_force_newtons` | `f32` | N | `2200.0` | `>= 0` | Passive drag when throttle is near zero | Too high makes coasting impossible |
| `reverse_speed_threshold_mps` | `f32` | m/s | `1.25` | `>= 0` | Stop-then-reverse threshold | Too large blocks deliberate reversing |
| `limited_slip_load_bias` | `f32` | ratio | `0.55` | `0..=1` | Blend between authoring share and load share | Too high makes limited-slip behave like a spool |

## `SuspensionConfig`

| Field | Type | Unit | Default | Practical range | Effect | Common failure when mis-tuned |
| --- | --- | --- | --- | --- | --- | --- |
| `rest_length_m` | `f32` | m | `0.38` | `> 0` | Target suspension length around neutral ride height | Too large creates a monster-truck stance |
| `max_compression_m` | `f32` | m | `0.18` | `>= 0` | Allowed travel into bump | Too small bottoms harshly |
| `max_droop_m` | `f32` | m | `0.16` | `>= 0` | Allowed extension beyond rest | Too small loses contact over crests |
| `spring_strength_n_per_m` | `f32` | N/m | `29000.0` | `>= 0` | Spring force from compression | Too low wallows; too high chatters |
| `damper_strength_n_per_mps` | `f32` | N/(m/s) | `3600.0` | `>= 0` | Damping from suspension velocity | Too low bounces; too high feels stuck |
| `bump_stop_strength_n_per_m` | `f32` | N/m | `18000.0` | `>= 0` | Extra force once past min length | Too low slams through the floor; too high spikes violently |

## `TireGripConfig`

| Field | Type | Unit | Default | Practical range | Effect | Common failure when mis-tuned |
| --- | --- | --- | --- | --- | --- | --- |
| `longitudinal_grip` | `f32` | ratio | `1.35` | `>= 0` | Longitudinal grip limit relative to wheel load | Too low gives endless wheelspin |
| `lateral_grip` | `f32` | ratio | `1.15` | `>= 0` | Lateral grip limit relative to wheel load | Too high resists drift and curb recovery feels snappy |
| `longitudinal_stiffness` | `f32` | N per m/s-ish | `170.0` | `>= 0` | Passive correction along wheel forward | Too high jitters at low speed |
| `lateral_stiffness` | `f32` | N per m/s-ish | `460.0` | `>= 0` | Passive correction across the tire | Too high makes the car knife-edge into understeer |
| `lateral_response_exponent` | `f32` | exponent | `1.0` | `>= 0.5` | Shapes lateral response curve | Too high produces a dead center then sudden breakaway |
| `rolling_resistance_force_newtons` | `f32` | N | `32.0` | `>= 0` | Passive rolling drag | Too high kills coasting and top speed |
| `handbrake_lateral_multiplier` | `f32` | ratio | `0.42` | `>= 0` | Lateral grip scaling under handbrake | Too low makes the rear spin instantly |
| `handbrake_longitudinal_multiplier` | `f32` | ratio | `0.20` | `>= 0` | Longitudinal grip scaling under handbrake | Too high makes the handbrake ineffective |
| `low_speed_lateral_multiplier` | `f32` | ratio | `1.35` | `>= 1` typical | Extra lateral grip near stop | Too high causes parking-lot snapback |
| `nominal_load_newtons` | `f32` | N | `3500.0` | `> 0` | Reference load for sensitivity scaling | Wrong reference makes trucks and light cars feel similar |
| `load_sensitivity` | `f32` | exponent | `0.45` | `0..=1` | How strongly grip changes with load | Too high amplifies load transfer into sudden handling swings |

## `StabilityConfig`

| Field | Type | Unit | Default | Practical range | Effect | Common failure when mis-tuned |
| --- | --- | --- | --- | --- | --- | --- |
| `anti_roll_force_n_per_ratio` | `f32` | N | `3000.0` | `>= 0` | Anti-roll support from left-right compression delta | Too low tips over; too high lifts inside wheels |
| `park_hold_force_newtons` | `f32` | N | `4500.0` | `>= 0` | Tangent force budget for hill hold | Too low slides on slopes; too high sticks unnaturally |
| `park_hold_speed_threshold_mps` | `f32` | m/s | `0.65` | `>= 0` | Speed below which hill hold can engage | Too high drags while creeping |
| `low_speed_traction_boost` | `f32` | ratio | `1.25` | `>= 1` typical | Additional longitudinal support near stop | Too high hides real grip or brake tuning problems |
| `low_speed_traction_speed_threshold_mps` | `f32` | m/s | `3.0` | `>= 0` | Window where the boost is active | Too high affects normal cornering |
| `yaw_stability_torque_nm_per_radps` | `f32` | Nm per rad/s | `1500.0` | `>= 0` | High-speed yaw damping assist | Too high suppresses deliberate rotation |
| `yaw_stability_speed_threshold_mps` | `f32` | m/s | `8.0` | `>= 0` | Minimum speed for yaw damping | Too low fights parking maneuvers |
| `airborne_upright_torque_nm_per_rad` | `f32` | Nm per rad | `850.0` | `>= 0` | Upright assist while airborne | Too high snaps mid-air unrealistically |
| `drift_entry_ratio` | `f32` | ratio | `0.34` | `>= 0` | Drift threshold when not already drifting | Too low spams drift state changes |
| `drift_exit_ratio` | `f32` | ratio | `0.24` | `>= 0` | Exit threshold with hysteresis | Too high leaves drift state latched |

## `AerodynamicsConfig`

| Field | Type | Unit | Default | Practical range | Effect | Common failure when mis-tuned |
| --- | --- | --- | --- | --- | --- | --- |
| `drag_force_per_speed_sq` | `f32` | N per (m/s)^2 | `1.05` | `>= 0` | Speed-squared drag | Too high caps top speed too early |
| `downforce_per_speed_sq` | `f32` | N per (m/s)^2 | `0.18` | `>= 0` | Speed-squared downforce along chassis down | Too high glues the car to the road and masks balance issues |

## `GroundVehicleSurface`

| Field | Type | Unit | Default | Effect |
| --- | --- | --- | --- | --- |
| `longitudinal_grip_scale` | `f32` | ratio | `1.0` | Multiplies forward traction and braking grip |
| `lateral_grip_scale` | `f32` | ratio | `1.0` | Multiplies lateral grip |
| `rolling_drag_scale` | `f32` | ratio | `1.0` | Multiplies passive rolling drag |
| `brake_scale` | `f32` | ratio | `1.0` | Multiplies explicit brake request force |

## `GroundVehicleDebugDraw`

| Field | Type | Default | Effect |
| --- | --- | --- | --- |
| `enabled` | `bool` | `false` | Master switch for all runtime gizmos |
| `draw_suspension` | `bool` | `true` | Draw suspension travel lines |
| `draw_contact_normals` | `bool` | `true` | Draw contact normals at grounded wheels |
| `draw_force_vectors` | `bool` | `true` | Draw longitudinal and lateral force arrows |
| `draw_slip_vectors` | `bool` | `true` | Draw contact-patch slip arrows |
