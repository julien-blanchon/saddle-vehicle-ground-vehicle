# Saddle Vehicle Ground Vehicle

Reusable ground-vehicle controller for Bevy with Avian3D-backed suspension sampling, wheel steering, configurable powertrain strategies, tire grip, stability helpers, wheel-visual sync, and crate-local verification examples.

The crate is designed as a toolkit for cars, trucks, utility vehicles, and track-style rigs. It owns chassis and wheel force generation, but it does not own cameras, HUD, damage, missions, or game-specific vehicle genres.

## Quick Start

```toml
[dependencies]
bevy = "0.18"
saddle-vehicle-ground-vehicle = { git = "https://github.com/julien-blanchon/saddle-vehicle-ground-vehicle" }
```

```rust,no_run
use avian3d::prelude::*;
use bevy::prelude::*;
use saddle_vehicle_ground_vehicle::{
    GroundVehicle, GroundVehiclePlugin, GroundVehicleWheel, GroundVehicleWheelVisual,
    VehicleIntent, WheelSide,
};

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum DemoState {
    #[default]
    Running,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .init_state::<DemoState>()
        .add_plugins(GroundVehiclePlugin::new(
            OnEnter(DemoState::Running),
            OnExit(DemoState::Running),
            FixedUpdate,
        ))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let vehicle = GroundVehicle::default();
    let chassis = commands
        .spawn((
            Name::new("Demo Chassis"),
            vehicle,
            VehicleIntent::default(),
            Collider::cuboid(1.8, 0.7, 4.2),
            Transform::from_xyz(0.0, 1.1, 0.0),
        ))
        .id();

    let wheel_visual = commands
        .spawn((
            Name::new("Front Left Wheel Visual"),
            Mesh3d(meshes.add(Cylinder::new(0.36, 0.24))),
            MeshMaterial3d(materials.add(StandardMaterial::default())),
            Transform::from_translation(Vec3::new(-0.82, 0.45, -1.25)),
        ))
        .id();

    commands.spawn((
        Name::new("Front Left Wheel"),
        GroundVehicleWheel::default_front(
            chassis,
            Vec3::new(-0.82, -0.15, -1.25),
            WheelSide::Left,
        ),
        GroundVehicleWheelVisual {
            visual_entity: wheel_visual,
            base_rotation: Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            ..default()
        },
    ));
}
```

For examples and crate-local labs, `GroundVehiclePlugin::default()` is the always-on entrypoint. It activates on `PostStartup`, never deactivates, and updates in `FixedUpdate`.

## Coordinate And Unit Conventions

- Distances: meters
- Velocities: meters per second
- Angles: radians
- Forces: newtons
- Torques: newton-meters
- Chassis forward: `Transform::forward()` which in Bevy is local `-Z`
- Chassis right: local `+X`
- Chassis up: local `+Y`
- Wheel `mount_point`: chassis-local position of the suspension origin
- Wheel visuals are separate from the physics chassis and are written in `PostUpdate`

## Public API

| Type | Purpose |
| --- | --- |
| `GroundVehiclePlugin` | Registers the runtime with injectable activate, deactivate, and update schedules |
| `GroundVehicleSystems` | Public ordering hooks: `InputAdaptation`, `Suspension`, `Steering`, `Powertrain`, `Grip`, `Stability`, `Telemetry`, `VisualSync` |
| `GroundVehicle` | Chassis-level authored config: mass, inertia, steering, powertrain, stability, aero |
| `VehicleIntent` | Generic driver or AI intent: signed drive, signed turn, brake, and auxiliary brake |
| `GroundVehicleWheel` | Wheel authoring data: location, axle, drive/steer/brake role, suspension, tire |
| `GroundVehicleWheelVisual` | Binding from wheel runtime state to a visible mesh entity |
| `GroundVehicleWheelState` | Per-wheel runtime contact, load, slip, force, steer, and spin state |
| `GroundVehicleTelemetry` | Chassis-level runtime speed, grounded-wheel, normal, engine RPM, and selected-gear aggregation |
| `GroundVehicleSurface` | Optional surface multipliers for grip, rolling drag, and braking |
| `GroundVehicleReset` | Marker component: insert to teleport the vehicle and zero its velocities and wheel state |
| `GroundVehicleDebugDraw` | Runtime gizmo toggles for suspension, contact, force, and slip vectors |
| `SteeringConfig`, `PowertrainConfig`, `EngineConfig`, `DriveModel`, `GearModel`, `AutomaticGearboxConfig`, `FixedGearConfig`, `DifferentialConfig`, `SuspensionConfig`, `TireGripConfig`, `MagicFormulaConfig`, `StabilityConfig`, `AerodynamicsConfig` | Tunable sub-configs used by authored chassis and wheel data |
| `GroundVehicleDriftPlugin`, `GroundVehicleDriftConfig`, `GroundVehicleDriftTelemetry`, `DriftStateChanged` | Optional drift helper layer for slip-based drift telemetry and drift state messages |
| `WheelGroundedChanged`, `VehicleBecameAirborne`, `VehicleLanded` | Core messages for gameplay reactions, UI, VFX, or tuning tools |

The crate intentionally does not expose internal solver scratch state, axle accumulators, or force-request bookkeeping.

## Powertrain Model

`PowertrainConfig` separates the power source from the delivery strategy:

- `engine`: torque curve and engine-braking behavior
- `drive_model`: how torque is distributed, currently `DriveModel::Axle` or `DriveModel::Track`
- `gear_model`: how ratio selection works, currently `GearModel::Automatic` or `GearModel::Fixed`
- `brake_force_newtons` / `auxiliary_brake_force_newtons`: explicit brake budgets

This keeps the input surface generic while letting road vehicles, multi-axle trucks, and track-drive rigs share the same chassis and tire systems.

## Optional Drift Helper

Drift telemetry is not part of the core runtime anymore.

Add the optional helper when a game or example actually wants drift state:

```rust,no_run
use saddle_vehicle_ground_vehicle::{
    GroundVehicleDriftConfig, GroundVehicleDriftPlugin, GroundVehiclePlugin,
};

App::new()
    .add_plugins((
        GroundVehiclePlugin::default(),
        GroundVehicleDriftPlugin::default(),
    ))
    .add_systems(Startup, |mut commands: Commands| {
        commands.spawn((
            Name::new("Drift-Capable Vehicle"),
            GroundVehicle::default(),
            VehicleIntent::default(),
            GroundVehicleDriftConfig::default(),
        ));
    });
```

Attach `GroundVehicleDriftConfig` to the same entity as `GroundVehicle`. The drift helper then writes `GroundVehicleDriftTelemetry` and emits `DriftStateChanged` when the drift state toggles.

## Supported Vehicle Styles

- Four-wheel road vehicles with Ackermann steering
- Rear-biased drift cars through auxiliary-brake shaping plus linear or Magic Formula tire response
- Long-travel off-road and utility vehicles
- Multi-axle cargo trucks
- Left/right track-drive or skid-steer style vehicles through `DriveModel::Track`
- Single-speed or automatic geared powertrains

## What The Crate Does Not Do

- Full clutch simulation, engine-audio playback, or drivetrain damage
- Tire temperature, wear, and full motorsport-grade multi-point tire fitting
- Full tread simulation for tracks
- Camera rigs, HUD, replay, or networking
- Damage, deformation, or mission-specific gameplay rules
- Genre presets in the core API

The old arcade/sim/off-road presets were intentionally removed from the core crate. Example-specific presets now live in `examples/support` where they do not constrain the reusable public API.

## Examples

All example apps include live `saddle-pane` tuning and on-screen controls. The example support crate also adds the optional drift helper so drift telemetry is available in the demos and lab.

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Minimal four-wheel hatchback on a flat handling pad | `cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_example_basic` |
| `multi_axle` | Six-wheel truck across bumps and uneven support | `cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_example_multi_axle` |
| `drift_tuning` | Rear-biased drift coupe using the Magic Formula tire path for controllable breakaway | `cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_example_drift_tuning` |
| `driving_demo` | Checkpoint-based canyon driving demo with a scripted tiltrotor escort from `saddle-vehicle-flight` | `cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_example_driving_demo` |
| `skid_steer` | Left/right track-drive steering for tank-like or tracked-style control | `cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_example_skid_steer` |
| `slope_stability` | Hill hold, anti-roll, and low-speed traction on ramps and off-camber surfaces | `cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_example_slope_stability` |

## Crate-Local Lab

The richer standalone verification app lives under `examples/lab`:

```bash
cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_lab
```

### E2E Scenarios

The lab includes 7 automated E2E scenarios powered by `saddle-bevy-e2e`. Each scenario resets a specific vehicle, applies scripted inputs, captures screenshots, and runs soft assertions. Run them with:

```bash
cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_lab --features e2e -- <scenario_name>
```

| Scenario | Vehicle | What it verifies |
|---|---|---|
| `ground_vehicle_smoke` | Compact car | Settles on ground, builds forward speed under throttle, stays out of drift |
| `ground_vehicle_braking` | Compact car | Builds speed then brakes to a stop, maintains ground contact, no wild yaw |
| `ground_vehicle_drivetrain` | Compact car | Upshifts under sustained throttle, engine RPM stays in valid range |
| `ground_vehicle_slope` | Rover | Holds position on an inclined ramp under brake, stays grounded, detects slope normal |
| `ground_vehicle_drift` | Drift coupe | Enters a drift with throttle + turn + aux brake, shows lateral movement, stays grounded |
| `ground_vehicle_skid_steer` | Skid vehicle | Yaws via left/right drive split (not wheel steer), keeps all wheels on ground |
| `ground_vehicle_multi_axle` | Cargo truck | Stays upright and grounded while crossing a bump course, no drift state |

Each scenario writes its output to `examples/e2e_output/<scenario_name>/`:
- `log.txt` — timestamped action log with pass/fail results
- `*.png` — screenshots at key moments (start, mid, end states)

Add `--handoff` to keep the app running after the scenario for interactive debugging:

```bash
cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_lab --features e2e -- ground_vehicle_smoke --handoff
```

### Resetting Vehicles

When teleporting a vehicle (e.g. for respawn or scenario reset), use `reset_vehicle_state` to flush the internal wheel/powertrain state immediately. Without this, stale suspension history causes a damper force spike on the first physics frame after teleport:

```rust
// In an exclusive system or Custom E2E action with &mut World:
*world.get_mut::<Transform>(chassis).unwrap() = new_transform;
*world.get_mut::<LinearVelocity>(chassis).unwrap() = LinearVelocity::ZERO;
*world.get_mut::<AngularVelocity>(chassis).unwrap() = AngularVelocity::ZERO;
ground_vehicle::reset_vehicle_state(world, chassis);
```

## BRP

`ground_vehicle_lab` uses BRP port `15712` by default to avoid collisions with other local Bevy apps. Override with `GROUND_VEHICLE_LAB_BRP_PORT` if needed.

```bash
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp app launch ground_vehicle_lab
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query ground_vehicle::components::GroundVehicleTelemetry
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query ground_vehicle::drift::GroundVehicleDriftTelemetry
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query ground_vehicle::components::GroundVehicleWheelState
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/ground_vehicle_lab.png
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp extras shutdown
```

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Tuning](docs/tuning.md)
- [Debugging](docs/debugging.md)
