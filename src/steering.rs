use crate::{
    GroundVehicle, GroundVehicleResolvedIntent, GroundVehicleWheel, GroundVehicleWheelState,
    SteeringMode, WheelSide,
};
use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;

#[derive(Clone, Copy, Default)]
struct AckermannAxleAccumulator {
    left_count: u32,
    right_count: u32,
    left_sum: Vec3,
    right_sum: Vec3,
    steer_weight_sum: f32,
}

pub(crate) fn speed_sensitive_factor(steering: crate::SteeringConfig, speed_mps: f32) -> f32 {
    if steering.speed_reduction_end_mps <= steering.speed_reduction_start_mps {
        return 1.0;
    }
    if speed_mps <= steering.speed_reduction_start_mps {
        return 1.0;
    }
    let t = ((speed_mps - steering.speed_reduction_start_mps)
        / (steering.speed_reduction_end_mps - steering.speed_reduction_start_mps))
        .clamp(0.0, 1.0);
    1.0 + (steering.minimum_speed_factor - 1.0) * t
}

pub(crate) fn ackermann_pair(
    base_angle_rad: f32,
    wheelbase_m: f32,
    track_width_m: f32,
    ackermann_ratio: f32,
) -> (f32, f32) {
    if base_angle_rad.abs() < f32::EPSILON
        || wheelbase_m <= 0.0
        || track_width_m <= 0.0
        || ackermann_ratio <= 0.0
    {
        return (base_angle_rad, base_angle_rad);
    }

    let steer_abs = base_angle_rad.abs();
    let turn_radius = wheelbase_m / steer_abs.tan().max(0.001);
    let inner = (wheelbase_m / (turn_radius - track_width_m * 0.5).max(0.01)).atan();
    let outer = (wheelbase_m / (turn_radius + track_width_m * 0.5)).atan();
    let sign = base_angle_rad.signum();
    let left = if sign >= 0.0 { inner } else { outer } * sign;
    let right = if sign >= 0.0 { outer } else { inner } * sign;
    let blend = ackermann_ratio.clamp(0.0, 1.0);
    (
        base_angle_rad.lerp(left, blend),
        base_angle_rad.lerp(right, blend),
    )
}

pub(crate) fn derive_ackermann_geometry<'a>(
    wheels: impl IntoIterator<Item = &'a GroundVehicleWheel>,
) -> Option<(f32, f32)> {
    let mut axles = [AckermannAxleAccumulator::default(); 8];

    for wheel in wheels {
        if matches!(wheel.side, WheelSide::Center) {
            continue;
        }

        let axle_slot = usize::from(wheel.axle).min(axles.len() - 1);
        let axle = &mut axles[axle_slot];
        axle.steer_weight_sum += wheel.steer_factor.abs();

        match wheel.side {
            WheelSide::Left => {
                axle.left_count += 1;
                axle.left_sum += wheel.mount_point;
            }
            WheelSide::Right => {
                axle.right_count += 1;
                axle.right_sum += wheel.mount_point;
            }
            WheelSide::Center => {}
        }
    }

    let (selected_index, selected_axle) = axles
        .iter()
        .enumerate()
        .filter(|(_, axle)| {
            axle.left_count > 0 && axle.right_count > 0 && axle.steer_weight_sum > 0.01
        })
        .max_by(|(_, a), (_, b)| a.steer_weight_sum.total_cmp(&b.steer_weight_sum))?;

    let left_center = selected_axle.left_sum / selected_axle.left_count as f32;
    let right_center = selected_axle.right_sum / selected_axle.right_count as f32;
    let steer_center_z = (left_center.z + right_center.z) * 0.5;
    let track_width_m = (right_center.x - left_center.x).abs();

    let mut other_axle_z_sum = 0.0;
    let mut other_axle_count = 0_u32;
    for (index, axle) in axles.iter().enumerate() {
        if index == selected_index || axle.left_count == 0 || axle.right_count == 0 {
            continue;
        }

        let left = axle.left_sum / axle.left_count as f32;
        let right = axle.right_sum / axle.right_count as f32;
        other_axle_z_sum += (left.z + right.z) * 0.5;
        other_axle_count += 1;
    }

    if other_axle_count == 0 {
        return None;
    }

    let wheelbase_m = (other_axle_z_sum / other_axle_count as f32 - steer_center_z).abs();
    (wheelbase_m > 0.01 && track_width_m > 0.01).then_some((wheelbase_m, track_width_m))
}

pub(crate) fn resolved_ackermann_geometry(
    steering: crate::SteeringConfig,
    chassis: Entity,
    all_wheels: &Query<&GroundVehicleWheel>,
) -> Option<(f32, f32)> {
    let derived =
        derive_ackermann_geometry(all_wheels.iter().filter(|wheel| wheel.chassis == chassis));
    match (
        steering.wheelbase_override_m,
        steering.track_width_override_m,
        derived,
    ) {
        (Some(wheelbase), Some(track_width), _) => Some((wheelbase, track_width)),
        (Some(wheelbase), None, Some((_, track_width))) => Some((wheelbase, track_width)),
        (None, Some(track_width), Some((wheelbase, _))) => Some((wheelbase, track_width)),
        (None, None, Some(geometry)) => Some(geometry),
        _ => None,
    }
}

pub(crate) fn update_steering_angles(
    time: Res<Time>,
    mut wheels: Query<(&GroundVehicleWheel, &mut GroundVehicleWheelState)>,
    all_wheels: Query<&GroundVehicleWheel>,
    chassis: Query<(
        &GroundVehicle,
        &GroundVehicleResolvedIntent,
        &LinearVelocity,
        &Transform,
    )>,
) {
    let dt = time.delta_secs().max(1.0 / 480.0);

    for (wheel, mut state) in &mut wheels {
        let Ok((vehicle, input, linear_velocity, transform)) = chassis.get(wheel.chassis) else {
            continue;
        };

        let target_angle = match vehicle.steering.mode {
            SteeringMode::Disabled => 0.0,
            SteeringMode::Road => {
                let forward_speed_mps = linear_velocity.0.dot(*transform.forward());
                let speed_factor =
                    speed_sensitive_factor(vehicle.steering, forward_speed_mps.abs());
                let steer_sign = input.turn.signum() * wheel.steer_factor.signum();
                let base_angle = vehicle.steering.max_angle_rad
                    * speed_factor
                    * input.turn.abs()
                    * wheel.steer_factor.abs()
                    * steer_sign;

                if let Some((wheelbase_m, track_width_m)) =
                    resolved_ackermann_geometry(vehicle.steering, wheel.chassis, &all_wheels)
                {
                    let (left, right) = ackermann_pair(
                        base_angle,
                        wheelbase_m,
                        track_width_m,
                        vehicle.steering.ackermann_ratio,
                    );
                    match wheel.side {
                        WheelSide::Left => left,
                        WheelSide::Right => right,
                        WheelSide::Center => base_angle,
                    }
                } else {
                    base_angle
                }
            }
        };

        let max_step = vehicle.steering.steer_rate_rad_per_sec.max(0.1) * dt;
        let delta = (target_angle - state.steer_angle_rad).clamp(-max_step, max_step);
        state.steer_angle_rad += delta;
    }
}
