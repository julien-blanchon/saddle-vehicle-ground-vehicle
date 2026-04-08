# Tuning

## Baseline Workflow

Tune vehicles in this order:

1. Set mass, inertia, and center of mass.
2. Set wheel placement and suspension travel.
3. Make the vehicle settle at a plausible ride height.
4. Tune drive and brake force.
5. Tune lateral grip and steering.
6. Add anti-roll, low-speed traction, yaw damping, and aero last.

If too many knobs seem to fight each other, return to a simple flat-pad test with:

- one surface type
- debug draw off
- assists reduced
- moderate speed
- no auxiliary brake

## Light Vs Heavy Vehicles

### Light road cars

Start with:

- lower `mass_kg`
- moderate `angular_inertia_kgm2`
- lower `anti_roll_force_n_per_ratio`
- faster steering rate
- lighter brake and drive budgets

Symptoms of overdoing it:

- too much snap rotation under steering lift
- curb hits bounce the whole body upward
- optional drift telemetry triggers constantly

### Heavy trucks and utility vehicles

Start with:

- higher `mass_kg`
- substantially higher pitch and roll inertia
- lower steer rate and max angle
- stronger service brakes
- more suspension travel or softer springs if off-road
- stronger anti-roll only after ride height is stable

Symptoms of under-support:

- visible side-to-side rocking after every steering correction
- inside wheels lifting on simple ramps
- rear axles losing contact on gentle crests

## Suspension

### Ride height

The neutral parked stance is controlled by the relationship between:

- chassis mass
- wheel count and load distribution
- `rest_length_m`
- `spring_strength_n_per_m`

If the parked vehicle sits too low:

- increase spring strength
- increase rest length
- reduce mass

Do not fix every sagging issue with bump-stop strength. Bump stops are for the last part of travel, not the normal ride height.

### Damping

`damper_strength_n_per_mps` is the first knob to touch when the car oscillates.

- Too low: repeated bobbing after landings or curb strikes
- Too high: the vehicle feels like it sticks or punches down into terrain

### Anti-roll

Use `anti_roll_force_n_per_ratio` after the basic spring/damper pass is believable.

- Too little anti-roll: lazy body roll, wheel lift on off-camber transitions
- Too much anti-roll: diagonal wheel unloading and an unnaturally rigid stance

## Steering

### Road vehicles

Primary knobs:

- `max_angle_rad`
- `steer_rate_rad_per_sec`
- `ackermann_ratio`
- `minimum_speed_factor`

Guidance:

- Increase `max_angle_rad` for low-speed rotation and drift setups.
- Reduce `steer_rate_rad_per_sec` for heavy trucks and high-speed stability.
- Keep `ackermann_ratio` moderate if wheelbase or track width overrides are approximate rather than exact.

### Skid steer

Use:

- `SteeringMode::Disabled`
- left/right `drive_side`
- `TrackDriveConfig::turn_split`

If the vehicle pirouettes too aggressively, reduce `turn_split` before weakening raw drive force.

## Grip And Drift

### Arcade-to-sim bias

More arcade:

- higher low-speed traction boost
- more airborne upright assist
- lower lateral stiffness with a gentle exponent
- stronger yaw damping

More sim-lite:

- reduce assists
- rely more on mass, load transfer, and tire grip
- keep auxiliary-brake grip multipliers closer to realistic tire behavior

### Drift tuning

A drift-friendly setup usually needs:

- RWD or rear-biased drive
- more steering lock
- lower rear `lateral_grip`
- reduced `auxiliary_brake_lateral_multiplier`
- lower rear `auxiliary_brake_longitudinal_multiplier`
- lower or softer yaw damping so the chassis can rotate

Typical progression:

1. tune normal cornering first
2. reduce rear lateral grip a little
3. increase steering lock
4. use auxiliary-brake multipliers to widen the breakaway window
5. only then adjust optional drift-helper thresholds

If the car instantly spins instead of holding a drift:

- rear lateral grip is probably too low
- auxiliary-brake multiplier is probably too aggressive
- or yaw damping is too weak for the speed range

## Slopes And Utility Vehicles

For predictable low-speed hill behavior:

- raise `park_hold_force_newtons`
- keep `park_hold_speed_threshold_mps` low enough to avoid dragging while creeping
- increase `low_speed_traction_boost` carefully
- keep center of mass low

If the vehicle chatters while stopped on a slope:

- reduce brake force slightly
- reduce longitudinal stiffness
- lower low-speed traction boost
- check for overly stiff springs combined with short travel

## Aero And Stability Assists

Aerodynamic drag should shape top speed and high-speed braking feel, not replace grip tuning.

- If the vehicle refuses to coast, drag is too high.
- If high-speed lane changes feel floaty, a small amount of downforce may help.
- If a drift build stops rotating as speed rises, downforce may be masking the rear grip reduction.

Yaw damping is a finishing pass:

- raise it when the vehicle oscillates after turn-in
- lower it when the vehicle refuses to rotate under deliberate input

## Common Tuning Patterns

### Hatchback baseline

- moderate steer lock
- mild Ackermann
- moderate front/rear lateral grip
- mild anti-roll
- little or no downforce

### Drift coupe

- more steering lock
- rear drive bias
- reduced rear lateral grip
- strong rear auxiliary-brake grip reduction
- lighter yaw damping

### Cargo truck

- more mass and inertia
- slow steering
- stronger brakes
- more anti-roll after springs are stable
- moderate aero drag, little downforce

### Rover or hill-climber

- low top speed
- strong hill hold
- generous low-speed traction support
- lower center of mass
- large wheel radius and longer suspension travel

## Verification Loop

Use the examples and lab as a repeatable tuning harness:

```bash
cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_example_basic
cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_example_drift_tuning
cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_example_multi_axle
cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_lab --features e2e -- ground_vehicle_smoke
cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_lab --features e2e -- ground_vehicle_drift
cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_lab --features e2e -- ground_vehicle_skid_steer
cargo run --manifest-path examples/Cargo.toml -p ground_vehicle_lab --features e2e -- ground_vehicle_slope
```

Keep screenshots, telemetry, and BRP inspection in the loop instead of tuning only from feel.
