mod components;
mod config;
mod debug;
mod drift;
mod drivetrain;
mod grip;
mod messages;
mod steering;
mod suspension;
mod systems;
mod visuals;

pub(crate) use components::{
    AxleAccumulator, GroundVehicleInternal, GroundVehicleResolvedIntent, GroundVehicleWheelInternal,
};
pub use components::{
    GroundVehicle, GroundVehicleDebugDraw, GroundVehicleReset, GroundVehicleSurface,
    GroundVehicleTelemetry, GroundVehicleWheel, GroundVehicleWheelState, GroundVehicleWheelVisual,
    VehicleIntent, WheelSide,
};
pub use config::{
    AerodynamicsConfig, AutomaticGearboxConfig, AxleDriveConfig, DifferentialConfig,
    DifferentialMode, DirectionChangeConfig, DirectionChangePolicy, DriveModel, EngineConfig,
    FixedGearConfig, GearModel, MagicFormulaConfig, PowertrainConfig, StabilityConfig,
    SteeringConfig, SteeringMode, SuspensionConfig, TireGripConfig, TireModel, TrackDriveConfig,
};
pub use drift::{
    DriftStateChanged, GroundVehicleDriftConfig, GroundVehicleDriftPlugin,
    GroundVehicleDriftSystems, GroundVehicleDriftTelemetry,
};
pub use messages::{VehicleBecameAirborne, VehicleLanded, WheelGroundedChanged};
pub use systems::reset_vehicle_state;

use bevy::{
    app::{FixedUpdate, PostStartup},
    ecs::schedule::common_conditions::resource_exists,
    ecs::{intern::Interned, schedule::ScheduleLabel},
    gizmos::{config::DefaultGizmoConfigGroup, gizmos::GizmoStorage},
    prelude::*,
    transform::TransformSystems,
};

#[derive(SystemSet, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum GroundVehicleSystems {
    InputAdaptation,
    Suspension,
    Steering,
    Powertrain,
    Grip,
    Stability,
    Telemetry,
    VisualSync,
}

#[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
struct NeverDeactivateSchedule;

pub struct GroundVehiclePlugin {
    pub activate_schedule: Interned<dyn ScheduleLabel>,
    pub deactivate_schedule: Interned<dyn ScheduleLabel>,
    pub update_schedule: Interned<dyn ScheduleLabel>,
}

impl GroundVehiclePlugin {
    pub fn new(
        activate_schedule: impl ScheduleLabel,
        deactivate_schedule: impl ScheduleLabel,
        update_schedule: impl ScheduleLabel,
    ) -> Self {
        Self {
            activate_schedule: activate_schedule.intern(),
            deactivate_schedule: deactivate_schedule.intern(),
            update_schedule: update_schedule.intern(),
        }
    }

    pub fn always_on(update_schedule: impl ScheduleLabel) -> Self {
        Self::new(PostStartup, NeverDeactivateSchedule, update_schedule)
    }
}

impl Default for GroundVehiclePlugin {
    fn default() -> Self {
        Self::always_on(FixedUpdate)
    }
}

impl Plugin for GroundVehiclePlugin {
    fn build(&self, app: &mut App) {
        if self.deactivate_schedule == NeverDeactivateSchedule.intern() {
            app.init_schedule(NeverDeactivateSchedule);
        }

        app.init_resource::<GroundVehicleDebugDraw>()
            .init_resource::<systems::GroundVehicleRuntime>()
            .add_message::<WheelGroundedChanged>()
            .add_message::<VehicleBecameAirborne>()
            .add_message::<VehicleLanded>()
            .register_type::<AerodynamicsConfig>()
            .register_type::<AutomaticGearboxConfig>()
            .register_type::<AxleDriveConfig>()
            .register_type::<DifferentialConfig>()
            .register_type::<DifferentialMode>()
            .register_type::<DirectionChangeConfig>()
            .register_type::<DirectionChangePolicy>()
            .register_type::<DriveModel>()
            .register_type::<EngineConfig>()
            .register_type::<FixedGearConfig>()
            .register_type::<GearModel>()
            .register_type::<GroundVehicle>()
            .register_type::<GroundVehicleDebugDraw>()
            .register_type::<GroundVehicleReset>()
            .register_type::<GroundVehicleSurface>()
            .register_type::<GroundVehicleTelemetry>()
            .register_type::<GroundVehicleWheel>()
            .register_type::<GroundVehicleWheelState>()
            .register_type::<GroundVehicleWheelVisual>()
            .register_type::<MagicFormulaConfig>()
            .register_type::<PowertrainConfig>()
            .register_type::<SteeringConfig>()
            .register_type::<SteeringMode>()
            .register_type::<StabilityConfig>()
            .register_type::<SuspensionConfig>()
            .register_type::<TireGripConfig>()
            .register_type::<TireModel>()
            .register_type::<TrackDriveConfig>()
            .register_type::<VehicleIntent>()
            .register_type::<WheelSide>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    GroundVehicleSystems::InputAdaptation,
                    GroundVehicleSystems::Suspension,
                    GroundVehicleSystems::Steering,
                    GroundVehicleSystems::Powertrain,
                    GroundVehicleSystems::Grip,
                    GroundVehicleSystems::Stability,
                    GroundVehicleSystems::Telemetry,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                systems::process_vehicle_resets.before(GroundVehicleSystems::InputAdaptation),
            )
            .add_systems(
                self.update_schedule,
                (
                    systems::sync_ground_vehicle_properties
                        .in_set(GroundVehicleSystems::InputAdaptation),
                    systems::sync_new_wheel_state.in_set(GroundVehicleSystems::InputAdaptation),
                    drivetrain::resolve_vehicle_intent
                        .in_set(GroundVehicleSystems::InputAdaptation),
                    (
                        suspension::reset_chassis_accumulators,
                        suspension::sample_wheels_and_apply_suspension,
                    )
                        .chain()
                        .in_set(GroundVehicleSystems::Suspension),
                    steering::update_steering_angles.in_set(GroundVehicleSystems::Steering),
                    (
                        drivetrain::update_powertrain_state,
                        drivetrain::resolve_wheel_force_requests,
                    )
                        .chain()
                        .in_set(GroundVehicleSystems::Powertrain),
                    grip::apply_tire_forces.in_set(GroundVehicleSystems::Grip),
                    (
                        systems::apply_stability_helpers,
                        systems::apply_aerodynamics,
                    )
                        .chain()
                        .in_set(GroundVehicleSystems::Stability),
                    (
                        systems::update_vehicle_telemetry,
                        systems::emit_wheel_grounded_messages,
                        systems::emit_vehicle_state_messages,
                    )
                        .chain()
                        .in_set(GroundVehicleSystems::Telemetry),
                )
                    .run_if(systems::runtime_is_active),
            )
            .configure_sets(
                PostUpdate,
                GroundVehicleSystems::VisualSync.before(TransformSystems::Propagate),
            )
            .add_systems(
                PostUpdate,
                (
                    visuals::sync_wheel_visuals.in_set(GroundVehicleSystems::VisualSync),
                    debug::draw_debug_gizmos
                        .after(GroundVehicleSystems::VisualSync)
                        .run_if(resource_exists::<GizmoStorage<DefaultGizmoConfigGroup, ()>>)
                        .run_if(systems::runtime_is_active),
                ),
            );
    }
}

#[cfg(test)]
#[path = "systems_tests.rs"]
mod systems_tests;

#[cfg(test)]
#[path = "drivetrain_tests.rs"]
mod drivetrain_tests;

#[cfg(test)]
#[path = "grip_tests.rs"]
mod grip_tests;
