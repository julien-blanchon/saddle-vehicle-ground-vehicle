# `ground_vehicle_lab`

Standalone showcase and verification app for the shared `ground_vehicle` crate.

## Run

```bash
cargo run -p ground_vehicle_lab
```

Keys:

- `1`: compact car
- `2`: drift coupe
- `3`: cargo truck
- `4`: skid vehicle
- `5`: slope rover
- `W/S`: throttle and reverse
- `A/D`: steering or turn demand
- `Space`: brake
- `Shift`: handbrake
- `R`: reset active vehicle

## E2E

```bash
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_smoke
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_braking
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_slope
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_drift
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_skid_steer
cargo run -p ground_vehicle_lab --features e2e -- ground_vehicle_multi_axle
```

## BRP

`ground_vehicle_lab` listens on BRP port `15712` by default so it does not collide with other local Bevy apps. Override with `GROUND_VEHICLE_LAB_BRP_PORT` if needed.

```bash
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp app launch ground_vehicle_lab
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query bevy_ecs::name::Name
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp world query ground_vehicle::components::GroundVehicleTelemetry
BRP_PORT=15712 uv run --active --project .codex/skills/bevy-brp/script brp extras screenshot /tmp/ground_vehicle_lab.png
```
