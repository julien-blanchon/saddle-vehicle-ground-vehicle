# Game Genre Guide

This crate is a generic ground-vehicle toolkit. The same plugin, components, and physics pipeline power everything from a realistic sim racer to a Mario Kart to a motorcycle. The game genre comes from **how you tune the numbers**, not from different code paths.

## The Realism Dial

Every vehicle sits somewhere on a spectrum from arcade to simulation. Three groups of parameters control where:

| Layer | What it controls | Arcade end | Simulation end |
|---|---|---|---|
| **Stability** | Can the vehicle tip/spin? | High upright torque, high yaw damping | Low or zero aids |
| **Tires** | How much grip, how predictable? | Linear model, high grip values | MagicFormula, realistic grip |
| **Powertrain** | How responsive? | Fixed gear, instant direction change | Multi-speed, stop-then-change |

## Stability — The Most Important Lever

`StabilityConfig` has three key fields that determine the "feel":

```rust
StabilityConfig {
    // Resists yaw rotation (spinning out). Higher = harder to spin.
    yaw_stability_torque_nm_per_radps: f32,

    // Self-rights the vehicle when ALL wheels are off the ground.
    airborne_upright_torque_nm_per_rad: f32,

    // Always-on upright correction — works every frame, grounded or not.
    // This is the "arcade mode" switch. Set to 0 for realistic physics.
    // Set high (5,000–20,000+) for vehicles that should never tip over.
    roll_upright_torque_nm_per_rad: f32,
}
```

### Choosing values

| Style | `yaw_stability` | `airborne_upright` | `roll_upright` |
|---|---|---|---|
| Sim racing | 600 | 400 | 0 |
| Realistic road car | 2,000 | 1,200 | 0 |
| GTA / open world | 4,500 | 6,000 | 8,000 |
| Kart / arcade | 350 | 3,000 | 5,000 |
| Motorcycle | 180 | 4,000 | 20,000 |

`roll_upright_torque_nm_per_rad` is the single most impactful parameter. At 0, the vehicle obeys real physics and can flip. At 20,000, it physically cannot tip over — perfect for bikes and arcade games.

## Tire Model — Grip vs Slide

Two tire models are available per wheel:

### Linear (default)

Predictable, proportional response. Grip never "breaks away" — the force just saturates. Great for arcade games, karts, and any vehicle where the player shouldn't have to manage tire limits.

```rust
TireGripConfig {
    model: TireModel::Linear,
    longitudinal_grip: 1.90,  // higher = more traction
    lateral_grip: 1.60,       // higher = more cornering
    low_speed_lateral_multiplier: 1.50,  // extra grip at low speed
    ..default()
}
```

### MagicFormula

Realistic grip curve with a peak followed by falloff. The tire has a limit — push past it and grip drops, causing oversteer or understeer. Used for sim racing and drift cars.

```rust
TireGripConfig {
    model: TireModel::MagicFormula,
    longitudinal_grip: 1.72,
    lateral_grip: 1.48,
    magic_formula: MagicFormulaConfig {
        lateral_peak_slip_angle_rad: 9.0_f32.to_radians(),
        ..default()
    },
    ..default()
}
```

### Grip value ranges

| Style | Longitudinal | Lateral | Model |
|---|---|---|---|
| Sim racing | 1.5–1.8 | 1.3–1.6 | MagicFormula |
| Road car | 1.4–1.6 | 1.1–1.3 | Linear |
| Kart | 1.9–2.0 | 1.5–1.9 | Linear |
| Off-road | 1.7 | 1.0–1.1 | Linear |
| Drift car rear | 1.0–1.2 | 0.7–0.9 | MagicFormula |

## Powertrain — Response Character

### Single-speed kart

Instant, direct, no gear hunting. Feels like an electric motor. Pair with `DirectionChangePolicy::Immediate` for kart-style instant reverse.

```rust
PowertrainConfig {
    engine: EngineConfig {
        peak_torque_nm: 210.0,
        redline_rpm: 9_000.0,
        ..default()
    },
    gear_model: GearModel::Fixed(FixedGearConfig {
        forward_ratio: 7.50,
        coupling_speed_mps: 0.5,  // engages almost instantly
        direction_change: DirectionChangeConfig {
            policy: DirectionChangePolicy::Immediate,
            ..default()
        },
        ..default()
    }),
    ..default()
}
```

### Multi-speed automatic

Realistic shift behavior. Higher `coupling_speed_mps` = more lag from standstill (torque converter feel). Lower = snappier launch.

```rust
PowertrainConfig {
    engine: EngineConfig {
        peak_torque_nm: 820.0,
        redline_rpm: 8_200.0,
        ..default()
    },
    gear_model: GearModel::Automatic(AutomaticGearboxConfig {
        forward_gear_count: 6,
        final_drive_ratio: 3.15,
        shift_up_rpm: 7_600.0,
        coupling_speed_mps: 2.0,
        ..default()
    }),
    ..default()
}
```

### Drivetrain layout

```rust
// Front-wheel drive — stable, predictable, understeers
DriveModel::Axle(AxleDriveConfig {
    differential: DifferentialConfig { mode: DifferentialMode::Open, .. },
    ..
})
// → Front wheels: drive_factor: 1.0, Rear wheels: drive_factor: 0.0

// Rear-wheel drive — more oversteer, better for drift/sim
DriveModel::Axle(AxleDriveConfig {
    differential: DifferentialConfig { mode: DifferentialMode::LimitedSlip, .. },
    ..
})
// → Front wheels: drive_factor: 0.0, Rear wheels: drive_factor: 1.0

// AWD — all wheels driven
// → All wheels: drive_factor: 1.0

// Tank / skid-steer — turns by driving left/right sides differently
DriveModel::Track(TrackDriveConfig { turn_split: 0.92, .. })
// → Steering mode: Disabled, all wheels: drive_factor: 1.0
```

## Surfaces — Environmental Variety

Attach `GroundVehicleSurface` to any static collider to change how vehicles behave on it:

```rust
// Boost pad (kart racing)
GroundVehicleSurface {
    longitudinal_grip_scale: 1.8,
    lateral_grip_scale: 1.4,
    ..default()
}

// Oil slick
GroundVehicleSurface {
    longitudinal_grip_scale: 0.35,
    lateral_grip_scale: 0.25,
    brake_scale: 0.30,
    ..default()
}

// Dirt / off-road
GroundVehicleSurface {
    longitudinal_grip_scale: 0.70,
    lateral_grip_scale: 0.55,
    rolling_drag_scale: 2.5,
    ..default()
}

// Grass
GroundVehicleSurface {
    longitudinal_grip_scale: 0.60,
    lateral_grip_scale: 0.50,
    rolling_drag_scale: 3.0,
    brake_scale: 0.55,
}
```

## Wheel Visuals — Decoupled from Physics

The physics always uses paired left/right wheels per axle (the anti-roll bar and suspension need this). But **visuals are completely separate** — you choose what the player sees:

| Vehicle type | Physics wheels | Visible wheels | How |
|---|---|---|---|
| Car | 4 (2 axles) | 4 | Normal — each wheel gets a `GroundVehicleWheelVisual` |
| Truck | 6 (3 axles) | 6 | Same |
| Motorcycle | 4 (2 axles) | 2 (centered) | Left wheels get visual with `visual_offset_local` to center; right wheels have no visual |

To hide a physics wheel visually, simply don't attach `GroundVehicleWheelVisual` to it. The physics runs the same either way.

To shift a wheel's visual position (e.g., centering a motorcycle wheel):

```rust
GroundVehicleWheelVisual {
    visual_entity,
    // Shift the visual from x=-0.10 to x=0 (center)
    visual_offset_local: Vec3::new(0.10, 0.0, 0.0),
    base_rotation: Quat::from_rotation_z(FRAC_PI_2),
    ..default()
}
```

## Complete Genre Recipes

### Sim Racing

High power, minimal aids, MagicFormula tires, stiff suspension, strong aero.

- Mass: 1,400–1,600 kg
- Torque: 500–900 Nm
- Tires: `TireModel::MagicFormula` on all wheels
- `roll_upright_torque_nm_per_rad: 0.0`
- `yaw_stability_torque_nm_per_radps: 400–800`
- Suspension: stiff springs (40,000+ N/m), short travel
- Aero: high downforce (1.0+ per speed^2)

### Arcade Kart

Light, snappy, forgiving, impossible to flip.

- Mass: 250–400 kg
- Torque: 150–250 Nm
- `GearModel::Fixed` with `DirectionChangePolicy::Immediate`
- Tires: `TireModel::Linear`, grip 1.8–2.0
- `roll_upright_torque_nm_per_rad: 5,000+`
- `airborne_upright_torque_nm_per_rad: 3,000+`
- Low drag for high top speed

### Open World (GTA-style)

Heavy, stable, easy to drive, survives stunt jumps.

- Mass: 1,600–2,000 kg
- Torque: 600–800 Nm
- `yaw_stability_torque_nm_per_radps: 3,000–5,000` (won't spin)
- `roll_upright_torque_nm_per_rad: 6,000–10,000` (won't flip)
- `airborne_upright_torque_nm_per_rad: 5,000+` (lands on wheels)
- Moderate grip, Linear model

### Motorcycle

Narrow body, 2 visible wheels, physically cannot tip.

- Mass: 180–250 kg
- Narrow chassis collider (0.3–0.5m wide)
- 4 physics wheels at ±0.10m, only 2 visible (centered)
- `roll_upright_torque_nm_per_rad: 15,000–25,000`
- High-revving engine (12,000+ RPM redline)
- Quick steering (5+ rad/s)
- `DifferentialMode::Spool` (locked rear)

### Drift Car

RWD, slippery rear tires, strong front grip for steering control.

- Mass: 1,200–1,400 kg
- Front: `TireModel::Linear`, lateral_grip ~1.1
- Rear: `TireModel::MagicFormula`, lateral_grip ~0.8
- `DifferentialMode::Spool` (locked rear forces both wheels to spin)
- `auxiliary_brake_force_newtons` high for handbrake-initiated drifts
- Add `GroundVehicleDriftConfig` + `GroundVehicleDriftPlugin` for drift state detection

### Heavy Truck

Slow, heavy, lots of torque, many axles.

- Mass: 4,000–6,000 kg
- 6 wheels (3 axles), front steered, rear two driven
- Low RPM engine (2,000–3,500 RPM redline), high torque (1,000+ Nm)
- `DirectionChangePolicy::StopThenChange`
- Slow steering rate (1.5 rad/s), narrow lock angle (20–25 deg)
- Heavy suspension (50,000+ N/m springs)

### Tank / Skid-Steer

No wheel steering — turns by driving sides at different speeds.

- `SteeringMode::Disabled`
- `DriveModel::Track(TrackDriveConfig { turn_split: 0.90, .. })`
- `GearModel::Fixed` with `DirectionChangePolicy::Immediate`
- All wheels: `steer_factor: 0.0`, `drive_factor: 1.0`
- Low lateral grip (1.0) for easier skid-turning

## Summary Table

| Genre | Mass | Torque | Drag | Grip | `roll_upright` | Tire Model | Gears |
|---|---|---|---|---|---|---|---|
| Sim racing | 1,480 kg | 820 Nm | 0.55 | 1.5–1.8 | 0 | MagicFormula | 6-speed auto |
| Road car | 1,350 kg | 620 Nm | 0.42 | 1.1–1.5 | 0 | Linear | 5-speed auto |
| GTA-style | 1,800 kg | 720 Nm | 0.55 | 1.3–1.5 | 8,000 | Linear | 5-speed auto |
| Kart | 350 kg | 210 Nm | 0.18 | 1.6–2.0 | 5,000 | Linear | Fixed |
| Motorcycle | 220 kg | 195 Nm | 0.14 | 1.4–1.7 | 20,000 | Linear | 6-speed auto |
| Drift car | 1,250 kg | 680 Nm | 0.45 | 0.8–1.1 rear | 0 | MagicFormula rear | 5-speed auto |
| Truck | 4,800 kg | 1,650 Nm | 0.90 | 1.1–1.5 | 0 | Linear | 6-speed auto |
| Tank | 2,100 kg | 1,150 Nm | 0.75 | 1.0–1.6 | 0 | Linear | Fixed |
