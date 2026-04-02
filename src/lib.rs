mod components;
mod config;
mod debug;
mod drivetrain;
mod grip;
mod messages;
mod steering;
mod suspension;
mod systems;
mod visuals;

pub(crate) use components::{
    AxleAccumulator, GroundVehicleInternal, GroundVehicleResolvedControl,
    GroundVehicleWheelInternal,
};
pub use components::{
    GroundVehicle, GroundVehicleControl, GroundVehicleDebugDraw, GroundVehicleSurface,
    GroundVehicleTelemetry, GroundVehicleWheel, GroundVehicleWheelState, GroundVehicleWheelVisual,
    WheelSide,
};
pub use config::{
    AerodynamicsConfig, DifferentialMode, DrivetrainConfig, ReversePolicy, StabilityConfig,
    SteeringConfig, SteeringMode, SuspensionConfig, TireGripConfig,
};
pub use messages::{DriftStateChanged, VehicleBecameAirborne, VehicleLanded, WheelGroundedChanged};

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
    Drivetrain,
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
            .add_message::<DriftStateChanged>()
            .register_type::<AerodynamicsConfig>()
            .register_type::<DifferentialMode>()
            .register_type::<DrivetrainConfig>()
            .register_type::<GroundVehicle>()
            .register_type::<GroundVehicleControl>()
            .register_type::<GroundVehicleDebugDraw>()
            .register_type::<GroundVehicleSurface>()
            .register_type::<GroundVehicleTelemetry>()
            .register_type::<GroundVehicleWheel>()
            .register_type::<GroundVehicleWheelState>()
            .register_type::<GroundVehicleWheelVisual>()
            .register_type::<ReversePolicy>()
            .register_type::<SteeringConfig>()
            .register_type::<SteeringMode>()
            .register_type::<StabilityConfig>()
            .register_type::<SuspensionConfig>()
            .register_type::<TireGripConfig>()
            .register_type::<WheelSide>()
            .add_systems(self.activate_schedule, systems::activate_runtime)
            .add_systems(self.deactivate_schedule, systems::deactivate_runtime)
            .configure_sets(
                self.update_schedule,
                (
                    GroundVehicleSystems::InputAdaptation,
                    GroundVehicleSystems::Suspension,
                    GroundVehicleSystems::Steering,
                    GroundVehicleSystems::Drivetrain,
                    GroundVehicleSystems::Grip,
                    GroundVehicleSystems::Stability,
                    GroundVehicleSystems::Telemetry,
                )
                    .chain(),
            )
            .add_systems(
                self.update_schedule,
                (
                    systems::sync_ground_vehicle_properties
                        .in_set(GroundVehicleSystems::InputAdaptation),
                    systems::sync_new_wheel_state.in_set(GroundVehicleSystems::InputAdaptation),
                    drivetrain::resolve_control_intent
                        .in_set(GroundVehicleSystems::InputAdaptation),
                    (
                        suspension::reset_chassis_accumulators,
                        suspension::sample_wheels_and_apply_suspension,
                    )
                        .chain()
                        .in_set(GroundVehicleSystems::Suspension),
                    steering::update_steering_angles.in_set(GroundVehicleSystems::Steering),
                    drivetrain::resolve_wheel_force_requests
                        .in_set(GroundVehicleSystems::Drivetrain),
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
