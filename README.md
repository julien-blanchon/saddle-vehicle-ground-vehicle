# Saddle Vehicle Ground Vehicle

Reusable ground-vehicle controller for Bevy with Avian3D-backed suspension sampling, steering, drivetrain, tire grip, stability helpers, telemetry, wheel-visual sync, and crate-local lab verification.

The crate is intended for game-ready cars, trucks, utility vehicles, and skid-steer or tracked-style rigs. It does not try to be a motorsport simulator and it does not own camera, UI, mission, or damage systems.

## Quick Start

```toml
[dependencies]
bevy = "0.18"
ground_vehicle = { path = "shared/vehicle/ground_vehicle" }
```

```rust,no_run
use avian3d::prelude::*;
use bevy::prelude::*;
use ground_vehicle::{
    GroundVehicle, GroundVehiclePlugin, GroundVehicleWheel, GroundVehicleWheelVisual, WheelSide,
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
| `GroundVehicleSystems` | Public ordering hooks: `InputAdaptation`, `Suspension`, `Steering`, `Drivetrain`, `Grip`, `Stability`, `Telemetry`, `VisualSync` |
| `GroundVehicle` | Chassis-level authored config: mass, inertia, steering, drivetrain, stability, aero |
| `GroundVehicleControl` | Input-agnostic driver intent: throttle, brake, steering, handbrake |
| `GroundVehicleWheel` | Wheel authoring data: location, axle, drive/steer/brake role, suspension, tire |
| `GroundVehicleWheelVisual` | Binding from wheel runtime state to a visible mesh entity |
| `GroundVehicleWheelState` | Per-wheel runtime contact, load, slip, force, steer, and spin state |
| `GroundVehicleTelemetry` | Chassis-level runtime speed, drift, grounded-wheel, and normal aggregation |
| `GroundVehicleSurface` | Optional surface multipliers for grip, rolling drag, and braking |
| `GroundVehicleDebugDraw` | Runtime gizmo toggles for suspension, contact, force, and slip vectors |
| `SteeringConfig`, `DrivetrainConfig`, `SuspensionConfig`, `TireGripConfig`, `StabilityConfig`, `AerodynamicsConfig` | Tunable sub-configs used by authored chassis and wheel data |
| `WheelGroundedChanged`, `VehicleBecameAirborne`, `VehicleLanded`, `DriftStateChanged` | Optional cross-crate messages for gameplay reactions, UI, VFX, or tuning tools |

The crate intentionally does not expose internal solver scratch state, axle accumulators, or force-request bookkeeping.

## Supported Vehicle Styles

- Four-wheel road vehicles with Ackermann steering
- Rear-biased drift cars through tire config and handbrake shaping
- Long-travel off-road and utility vehicles
- Multi-axle cargo trucks
- Left/right skid-steer or tracked-style vehicles with differential turning

## What The Crate Does Not Do

- Gearbox, clutch, RPM, or engine-audio simulation
- Tire temperature, wear, or detailed brush/Pacejka tire models
- Full tread simulation for tracks
- Camera rigs, HUD, replay, or networking
- Damage, deformation, or mission-specific gameplay rules

## Examples

| Example | Purpose | Run |
| --- | --- | --- |
| `basic` | Minimal four-wheel hatchback on a flat handling pad | `cargo run -p ground_vehicle --example basic` |
| `multi_axle` | Six-wheel truck across bumps and uneven support | `cargo run -p ground_vehicle --example multi_axle` |
| `drift_tuning` | Rear-biased drift coupe with handbrake-friendly lateral grip shaping | `cargo run -p ground_vehicle --example drift_tuning` |
| `skid_steer` | Left/right drive-group steering for tank-like or tracked-style control | `cargo run -p ground_vehicle --example skid_steer` |
| `slope_stability` | Hill hold, anti-roll, and low-speed traction on ramps and off-camber surfaces | `cargo run -p ground_vehicle --example slope_stability` |

## Crate-Local Lab

The richer standalone verification app lives under `shared/vehicle/ground_vehicle/examples/lab`:

```bash
cargo run -p ground_vehicle_lab
```

E2E scenarios:

```bash
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_smoke
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_braking
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_slope
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_drift
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_skid_steer
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_multi_axle
```

## BRP

Useful BRP commands against the lab:

`ground_vehicle_lab` uses BRP port `15712` by default to avoid collisions with other local Bevy apps. Override with `GROUND_VEHICLE_LAB_BRP_PORT` if your environment needs a different port.

```bash
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp app launch ground_vehicle_lab
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query ground_vehicle::components::GroundVehicleTelemetry
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query ground_vehicle::components::GroundVehicleWheelState
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/ground_vehicle_lab.png
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp extras shutdown
```

## More Docs

- [Architecture](docs/architecture.md)
- [Configuration](docs/configuration.md)
- [Tuning](docs/tuning.md)
- [Debugging](docs/debugging.md)
