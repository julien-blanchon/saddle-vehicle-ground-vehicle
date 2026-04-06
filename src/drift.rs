use crate::{GroundVehicleTelemetry, GroundVehicleWheel, GroundVehicleWheelState, WheelSide};
use bevy::{
    app::FixedUpdate,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    prelude::*,
};

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component, Debug)]
#[require(GroundVehicleDriftInternal, GroundVehicleDriftTelemetry)]
pub struct GroundVehicleDriftConfig {
    pub entry_ratio: f32,
    pub exit_ratio: f32,
    pub minimum_forward_speed_mps: f32,
}

impl Default for GroundVehicleDriftConfig {
    fn default() -> Self {
        Self {
            entry_ratio: 0.34,
            exit_ratio: 0.24,
            minimum_forward_speed_mps: 5.0,
        }
    }
}

#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component, Debug)]
pub struct GroundVehicleDriftTelemetry {
    pub drift_ratio: f32,
    pub drifting: bool,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct DriftStateChanged {
    pub chassis: Entity,
    pub drifting: bool,
    pub drift_ratio: f32,
}

#[derive(Component, Debug, Clone, Copy, Default)]
struct GroundVehicleDriftInternal {
    was_drifting: bool,
}

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GroundVehicleDriftSystems {
    Telemetry,
    Messages,
}

pub struct GroundVehicleDriftPlugin {
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl GroundVehicleDriftPlugin {
    pub fn new(update_schedule: impl ScheduleLabel) -> Self {
        Self {
            update_schedule: update_schedule.intern(),
        }
    }
}

impl Default for GroundVehicleDriftPlugin {
    fn default() -> Self {
        Self::new(FixedUpdate)
    }
}

impl Plugin for GroundVehicleDriftPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DriftStateChanged>()
            .register_type::<GroundVehicleDriftConfig>()
            .register_type::<GroundVehicleDriftTelemetry>()
            .configure_sets(
                self.update_schedule,
                (
                    GroundVehicleDriftSystems::Telemetry,
                    GroundVehicleDriftSystems::Messages,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    update_drift_telemetry.in_set(GroundVehicleDriftSystems::Telemetry),
                    emit_drift_messages.in_set(GroundVehicleDriftSystems::Messages),
                ),
            );
    }
}

fn update_drift_telemetry(
    mut chassis: Query<(
        Entity,
        &GroundVehicleDriftConfig,
        &GroundVehicleTelemetry,
        &mut GroundVehicleDriftTelemetry,
        &GroundVehicleDriftInternal,
    )>,
    wheels: Query<(&GroundVehicleWheel, &GroundVehicleWheelState)>,
) {
    for (entity, config, telemetry, mut drift, internal) in &mut chassis {
        let mut drift_sum = 0.0;
        let mut drift_count = 0_u32;

        for (wheel, state) in &wheels {
            if wheel.chassis != entity {
                continue;
            }
            if wheel.drive_factor <= 0.0
                && wheel.auxiliary_brake_factor <= 0.0
                && !matches!(wheel.side, WheelSide::Left | WheelSide::Right)
            {
                continue;
            }

            drift_sum += (state.lateral_speed_mps.abs()
                / (state.longitudinal_speed_mps.abs() + 2.0))
                .clamp(0.0, 3.0);
            drift_count += 1;
        }

        let drift_ratio = if drift_count > 0 {
            drift_sum / drift_count as f32
        } else {
            0.0
        };
        let drift_threshold = if internal.was_drifting {
            config.exit_ratio
        } else {
            config.entry_ratio
        };

        drift.drift_ratio = drift_ratio;
        drift.drifting = !telemetry.airborne
            && telemetry.forward_speed_mps.abs() > config.minimum_forward_speed_mps
            && drift_ratio >= drift_threshold;
    }
}

fn emit_drift_messages(
    mut chassis: Query<(
        Entity,
        &GroundVehicleDriftTelemetry,
        &mut GroundVehicleDriftInternal,
    )>,
    writer: Option<MessageWriter<DriftStateChanged>>,
) {
    let Some(mut writer) = writer else {
        return;
    };

    for (entity, telemetry, mut internal) in &mut chassis {
        if telemetry.drifting != internal.was_drifting {
            writer.write(DriftStateChanged {
                chassis: entity,
                drifting: telemetry.drifting,
                drift_ratio: telemetry.drift_ratio,
            });
        }

        internal.was_drifting = telemetry.drifting;
    }
}
