# Architecture

## Layering

`ground_vehicle` keeps authored data, runtime state, force generation, and visual sync separated.

Public authored/runtime surface:

1. `components.rs`
2. `config.rs`
3. `messages.rs`
4. `drift.rs`
5. `lib.rs`

Internal simulation modules:

1. `suspension.rs`
2. `steering.rs`
3. `drivetrain.rs`
4. `grip.rs`
5. `systems.rs`
6. `visuals.rs`
7. `debug.rs`

Avian3D provides rigid bodies, spatial queries, mass properties, and force application. The crate owns the vehicle model itself.

## Entity Model

### Chassis entity

The chassis is the rigid-body root:

- `GroundVehicle`
- `VehicleIntent`
- `GroundVehicleTelemetry`
- Avian rigid-body components
- chassis collider and authored transform

Optional helper layers attach additional components such as `GroundVehicleDriftConfig` and `GroundVehicleDriftTelemetry`.

### Wheel entities

Each wheel is a lightweight authored/runtime record:

- `GroundVehicleWheel`
- `GroundVehicleWheelState`
- optional `GroundVehicleWheelVisual`

Wheels do not need to be rigid bodies. They act as suspension probes plus cached runtime state.

### Visual wheel entities

Visible wheel meshes are separate entities referenced by `GroundVehicleWheelVisual`. The crate writes their transforms in `PostUpdate` after simulation has settled. Visual sync never mutates physics state.

## Force Flow

```text
VehicleIntent
  -> clamp and resolve direction-change policy
  -> cast each wheel suspension probe
  -> accumulate chassis support loads
  -> resolve steering angles
  -> estimate engine RPM and active gear from driven-wheel speed
  -> resolve per-wheel drive and brake requests from the configured powertrain
  -> compute contact-patch longitudinal and lateral forces
  -> apply anti-roll / hill-hold / yaw / airborne assists
  -> apply aerodynamic drag and downforce
  -> aggregate chassis telemetry
  -> optional drift helper reads telemetry + wheel state
  -> sync visual wheels
```

## Schedule Ordering

`GroundVehicleSystems` is public and chained in this order on the injected update schedule:

1. `InputAdaptation`
2. `Suspension`
3. `Steering`
4. `Powertrain`
5. `Grip`
6. `Stability`
7. `Telemetry`

`VisualSync` runs separately in `PostUpdate` before `TransformSystems::Propagate`.

The optional drift helper uses its own `GroundVehicleDriftSystems` set and is expected to run after the core telemetry pass on the same fixed schedule.

## Fixed-Step Assumptions

The default plugin uses `FixedUpdate` because suspension and tire forces are much easier to tune with a stable step. Consumers can still inject another fixed schedule if their app already uses a custom stepping model.

Examples and the lab set `Time::<Fixed>::from_hz(60.0)`.

## Suspension Model

Each wheel performs a sphere shapecast along the suspension travel axis:

- origin: `GroundVehicleWheel::mount_point` transformed into world space
- direction: chassis local down projected into world
- shape: sphere with wheel radius
- range: `SuspensionConfig::max_length()`

On a valid hit the runtime computes:

- clamped suspension length
- compression and suspension velocity
- spring + damper + bump-stop support force
- grounded state, contact point, and normal

Support force is applied at the contact point on the chassis rigid body.

## Steering

Steering is purely about wheel angle resolution:

- `SteeringConfig` owns steering mode, max angle, rate limit, Ackermann blend, and speed reduction
- Ackermann geometry comes from explicit overrides when provided and otherwise falls back to the wheel layout on the chassis
- `SteeringMode::Disabled` is the escape hatch for track-drive or non-steering wheel layouts

The steering stage does not decide how torque is split. That is handled by the powertrain stage.

## Powertrain

`PowertrainConfig` splits torque production from delivery policy:

- `engine`: authored torque curve and engine-braking behavior
- `drive_model`: how torque is distributed
- `gear_model`: how ratio selection works
- explicit primary and auxiliary brake budgets

Current drive-model strategies:

- `DriveModel::Axle`: conventional axle-style torque delivery using authored wheel drive weights plus `DifferentialConfig`
- `DriveModel::Track`: left/right track-drive delivery using turn-demand splitting plus `DifferentialConfig`

Current gear-model strategies:

- `GearModel::Automatic`: automatic forward gears plus reverse handling and shift thresholds
- `GearModel::Fixed`: single fixed forward/reverse ratio with optional stop-then-change or immediate direction changes

This is the main decoupling from the older car-shaped runtime. Input is generic, and the powertrain stage chooses how to interpret it.

## Grip Model

The tire model is intentionally game-ready rather than study-level:

- per-wheel angular inertia and slip-ratio estimation
- separate longitudinal and lateral stiffness
- separate grip limits
- simple load-sensitivity scaling
- friction-circle clamp so combined force stays sane
- low-speed traction helper
- optional auxiliary-brake grip shaping
- optional Magic Formula response curves for both longitudinal and lateral force shaping
- optional `GroundVehicleSurface` multipliers from the contacted entity or its rigid-body owner

This gives a stable tuning surface for arcade-to-sim-lite handling without pretending to be a full tire research model.

## Stability Helpers

The crate layers pragmatic helpers on top of the wheel forces:

- anti-roll from left/right compression delta per axle
- hill-hold style brake assist at near-zero speed
- yaw-rate damping above a speed threshold
- airborne upright torque for arcade-friendly recovery
- aerodynamic drag and optional downforce

All of these are opt-in or tunable through config instead of hardcoded behavior.

## Runtime Outputs

Core outputs:

- `GroundVehicleWheelState` per wheel
- `GroundVehicleTelemetry` per chassis
- messages for wheel grounded transitions, airborne state, and landings

Optional drift outputs:

- `GroundVehicleDriftTelemetry`
- `DriftStateChanged`

Those surfaces are intended for UI, VFX, audio, telemetry overlays, BRP inspection, and E2E assertions.

## Testing Strategy

The crate verifies three layers:

- pure math and policy tests for suspension, steering, torque split, and direction-change rules
- lightweight Bevy app tests for plugin wiring and runtime state updates
- crate-local examples and a richer crate-local lab with E2E scenarios and screenshots
