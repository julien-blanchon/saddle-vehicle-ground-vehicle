use crate::{GroundVehicleWheel, GroundVehicleWheelState, GroundVehicleWheelVisual};
use bevy::prelude::*;

pub(crate) fn sync_wheel_visuals(
    wheels: Query<
        (
            &GroundVehicleWheel,
            &GroundVehicleWheelState,
            &GroundVehicleWheelVisual,
        ),
        Changed<GroundVehicleWheelState>,
    >,
    mut transforms: ParamSet<(Query<&Transform>, Query<&mut Transform>)>,
) {
    for (wheel, state, visual) in &wheels {
        let chassis_transform = {
            let chassis = transforms.p0();
            let Ok(chassis_transform) = chassis.get(wheel.chassis).copied() else {
                continue;
            };
            chassis_transform
        };
        let mut visuals = transforms.p1();
        let Ok(mut visual_transform) = visuals.get_mut(visual.visual_entity) else {
            continue;
        };

        let local_translation = wheel.mount_point
            + Vec3::NEG_Y * state.suspension_length_m
            + visual.visual_offset_local;
        let steering_rotation = Quat::from_axis_angle(
            visual.steering_axis_local.normalize_or_zero(),
            state.steer_angle_rad,
        );
        let spin_rotation = Quat::from_axis_angle(
            visual.rolling_axis_local.normalize_or_zero(),
            state.spin_angle_rad,
        );
        let local_rotation = steering_rotation * spin_rotation * visual.base_rotation;

        visual_transform.translation = chassis_transform.transform_point(local_translation);
        visual_transform.rotation = chassis_transform.rotation * local_rotation;
    }
}
