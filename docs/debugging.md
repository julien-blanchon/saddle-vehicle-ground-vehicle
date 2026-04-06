# Debugging

## First Values To Inspect

When a vehicle feels wrong, inspect these in order:

1. `ground_vehicle::components::GroundVehicleTelemetry`
2. `ground_vehicle::components::GroundVehicleWheelState`
3. `ground_vehicle::components::GroundVehicle`
4. `ground_vehicle::components::VehicleIntent`
5. `ground_vehicle::components::GroundVehicleSurface` on the contacted ground

Most real issues show up quickly in:

- grounded wheel count
- suspension length and force
- longitudinal and lateral wheel speeds
- longitudinal and lateral tire forces
- optional drift ratio from `GroundVehicleDriftTelemetry`
- average ground normal

## Common Failure Symptoms

| Symptom | Likely cause | Inspect first |
| --- | --- | --- |
| Vehicle spawns and explodes upward | collider overlap, too-short suspension, or huge spring strength | wheel `suspension_length_m`, chassis transform, support surface placement |
| Vehicle never moves under drive input | no driven wheels, zero drive force, or all wheels ungrounded | `drive_factor`, `grounded`, `longitudinal_force_newtons` |
| Vehicle slides downhill while braking | hill hold too weak or low-speed grip/brake scaling too low | telemetry speed, `park_hold_force_newtons`, surface `brake_scale` |
| Vehicle rolls over easily | center of mass too high or anti-roll too weak | `center_of_mass_offset`, anti-roll config, wheel compression delta |
| Optional drift state never enters | rear grip too high or auxiliary-brake shaping too mild | `GroundVehicleDriftTelemetry`, rear wheel lateral force, and forward speed |
| Steering feels fine at low speed but dead at medium speed | speed reduction start/end or min factor too aggressive | `SteeringConfig` speed reduction fields |
| Multi-axle truck jitters over bumps | too much spring or damping, or wheelbase bumps too sharp for the travel | per-wheel suspension force and contact state |
| Wheel visuals do not match runtime contact | wrong `GroundVehicleWheelVisual` binding or base rotation | wheel visual entity transform vs wheel state |

## Gizmos

Enable `GroundVehicleDebugDraw` to inspect:

- suspension lines
- contact normals
- longitudinal force arrows
- lateral force arrows
- slip vectors

Recommended workflow:

1. Start with only suspension and contact normals.
2. Add force vectors once ride height and contact look correct.
3. Add slip vectors for drift or braking diagnostics.

Force and slip arrows are most useful when the scene is simple and only one vehicle is active.

## BRP Workflows

Run the crate-local lab in foreground:

```bash
cargo run -p ground_vehicle_lab
```

The lab listens on BRP port `15712` by default so it does not collide with other local Bevy apps. Override with `GROUND_VEHICLE_LAB_BRP_PORT` if needed.

Or launch it through BRP:

```bash
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp app launch ground_vehicle_lab
```

Useful queries:

```bash
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query ground_vehicle::components::GroundVehicleTelemetry
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query ground_vehicle::components::GroundVehicleWheelState
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query ground_vehicle::components::GroundVehicle ground_vehicle::components::VehicleIntent
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query ground_vehicle::drift::GroundVehicleDriftTelemetry
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/ground_vehicle_lab.png
```

Use BRP to:

- confirm which wheels are grounded
- compare front/rear or left/right tire force balance
- inspect surface multipliers on ramps or pads
- verify visual wheel transforms against wheel runtime state
- freeze on a bad frame and capture a screenshot

## E2E Scenarios

The crate-local lab ships these scenarios:

- `ground_vehicle_smoke`
- `ground_vehicle_braking`
- `ground_vehicle_slope`
- `ground_vehicle_drift`
- `ground_vehicle_skid_steer`
- `ground_vehicle_multi_axle`

Run them with:

```bash
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_smoke
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_braking
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_slope
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_drift
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_skid_steer
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_multi_axle
```

Use them as a regression loop after tuning changes:

- smoke: spawn, settle, and accelerate
- braking: stop distance and force response
- slope: hill hold and contact stability
- drift: optional drift-helper posture and telemetry
- skid steer: left/right drive-group turning without wheel steer angles
- multi-axle: uneven support stability

## Deliberate Failure Cases Worth Reproducing

- full brake while turning
- auxiliary-brake entry at speed
- one side of the vehicle on a curb or small ramp
- cresting a hill and landing
- reversing uphill
- one wheel airborne while the opposite wheel is fully compressed

If the crate survives those without NaNs, explosions, or broken visuals, the tuning surface is usually in a good place.
