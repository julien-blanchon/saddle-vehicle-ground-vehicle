use avian3d::prelude::{AngularVelocity, LinearVelocity};
use bevy::prelude::*;
use ground_vehicle::GroundVehicleTelemetry;
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};

use crate::{DrivingDemoPlayer, DrivingDemoProgress};
use ground_vehicle_example_support::ScriptedControlOverride;

#[derive(Resource, Clone, Copy)]
struct DrivingDemoPlayerEntity(Entity);

#[derive(Clone, Copy)]
struct GateRun {
    start: Vec3,
    target: Vec3,
}

const CHECKPOINT_ONE_RUN: GateRun = GateRun {
    start: Vec3::new(-32.0, 1.18, -10.0),
    target: Vec3::new(-32.0, 1.18, -40.0),
};

const CHECKPOINT_TWO_RUN: GateRun = GateRun {
    start: Vec3::new(-8.0, 1.18, -62.0),
    target: Vec3::new(20.0, 1.18, -62.0),
};

const CHECKPOINT_THREE_RUN: GateRun = GateRun {
    start: Vec3::new(34.0, 1.18, -26.0),
    target: Vec3::new(34.0, 1.18, 6.0),
};

const CHECKPOINT_FOUR_RUN: GateRun = GateRun {
    start: Vec3::new(18.0, 1.18, 42.0),
    target: Vec3::new(-10.0, 1.18, 42.0),
};

pub fn list_scenarios() -> Vec<&'static str> {
    vec!["driving_demo_checkpoint_lap"]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "driving_demo_checkpoint_lap" => Some(checkpoint_lap()),
        _ => None,
    }
}

fn checkpoint_lap() -> Scenario {
    Scenario::builder("driving_demo_checkpoint_lap")
        .description(
            "Reset the checkpoint runner and drive short, deterministic approaches through all four \
             gates to verify checkpoint progression, lap completion, and timing in the actual \
             driving demo example.",
        )
        .then(Action::Custom(Box::new(|world: &mut World| {
            let player = player_entity(world).expect("driving demo player should exist");
            world.insert_resource(DrivingDemoPlayerEntity(player));
            reset_progress(world);
            reset_player(world, player, CHECKPOINT_ONE_RUN.start, CHECKPOINT_ONE_RUN.target);
            set_player_override(world, player, None);
        })))
        .then(Action::WaitFrames(10))
        .then(Action::WaitUntil {
            label: "checkpoint runner settled on the grid".into(),
            condition: Box::new(|world| {
                player_telemetry(world).is_some_and(|telemetry| {
                    telemetry.grounded_wheels >= 4 && telemetry.speed_mps < 0.3
                })
            }),
            max_frames: 240,
        })
        .then(Action::Screenshot("driving_demo_grid".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let player = world.resource::<DrivingDemoPlayerEntity>().0;
            set_player_override(world, player, throttle_run(0.85));
        })))
        .then(Action::WaitUntil {
            label: "checkpoint 1 cleared".into(),
            condition: Box::new(|world| world.resource::<DrivingDemoProgress>().next_checkpoint == 1),
            max_frames: 240,
        })
        .then(assertions::custom(
            "checkpoint runner reached gate 1 with useful speed",
            |world| {
                player_telemetry(world).is_some_and(|telemetry| {
                    telemetry.speed_mps > 3.0 && telemetry.grounded_wheels >= 2
                })
            },
        ))
        .then(Action::Screenshot("driving_demo_checkpoint_1".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let player = world.resource::<DrivingDemoPlayerEntity>().0;
            reset_player(world, player, CHECKPOINT_TWO_RUN.start, CHECKPOINT_TWO_RUN.target);
            set_player_override(world, player, throttle_run(0.80));
        })))
        .then(Action::WaitFrames(10))
        .then(Action::WaitUntil {
            label: "checkpoint 2 cleared".into(),
            condition: Box::new(|world| world.resource::<DrivingDemoProgress>().next_checkpoint == 2),
            max_frames: 240,
        })
        .then(Action::Custom(Box::new(|world: &mut World| {
            let player = world.resource::<DrivingDemoPlayerEntity>().0;
            reset_player(
                world,
                player,
                CHECKPOINT_THREE_RUN.start,
                CHECKPOINT_THREE_RUN.target,
            );
            set_player_override(world, player, throttle_run(0.80));
        })))
        .then(Action::WaitFrames(10))
        .then(Action::WaitUntil {
            label: "checkpoint 3 cleared".into(),
            condition: Box::new(|world| world.resource::<DrivingDemoProgress>().next_checkpoint == 3),
            max_frames: 240,
        })
        .then(Action::Custom(Box::new(|world: &mut World| {
            let player = world.resource::<DrivingDemoPlayerEntity>().0;
            reset_player(
                world,
                player,
                CHECKPOINT_FOUR_RUN.start,
                CHECKPOINT_FOUR_RUN.target,
            );
            set_player_override(world, player, throttle_run(0.80));
        })))
        .then(Action::WaitFrames(10))
        .then(Action::WaitUntil {
            label: "full checkpoint lap completed".into(),
            condition: Box::new(|world| world.resource::<DrivingDemoProgress>().laps_completed >= 1),
            max_frames: 240,
        })
        .then(assertions::custom("best lap time recorded", |world| {
            world.resource::<DrivingDemoProgress>().best_lap_seconds.is_some()
        }))
        .then(assertions::custom(
            "checkpoint loop reset to gate 1 after lap",
            |world| {
                let progress = world.resource::<DrivingDemoProgress>();
                progress.laps_completed >= 1 && progress.next_checkpoint == 0
            },
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let player = world.resource::<DrivingDemoPlayerEntity>().0;
            set_player_override(world, player, None);
        })))
        .then(Action::Screenshot("driving_demo_finish".into()))
        .then(Action::WaitFrames(1))
        .then(inspect::log_resource::<DrivingDemoProgress>(
            "driving_demo_progress",
        ))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "driving_demo_telemetry",
        ))
        .then(assertions::log_summary("driving_demo_checkpoint_lap summary"))
        .build()
}

fn player_entity(world: &mut World) -> Option<Entity> {
    let mut query = world.query_filtered::<Entity, With<DrivingDemoPlayer>>();
    query.iter(world).next()
}

fn player_telemetry(world: &World) -> Option<&GroundVehicleTelemetry> {
    let player = world.resource::<DrivingDemoPlayerEntity>().0;
    world.get::<GroundVehicleTelemetry>(player)
}

fn throttle_run(drive: f32) -> Option<ground_vehicle::VehicleIntent> {
    Some(ground_vehicle::VehicleIntent { drive, ..default() })
}

fn set_player_override(
    world: &mut World,
    player: Entity,
    override_intent: Option<ground_vehicle::VehicleIntent>,
) {
    if let Some(mut scripted_override) = world.get_mut::<ScriptedControlOverride>(player) {
        scripted_override.0 = override_intent;
    }
}

fn reset_progress(world: &mut World) {
    let now = world.resource::<Time>().elapsed_secs();
    let mut progress = world.resource_mut::<DrivingDemoProgress>();
    *progress = DrivingDemoProgress {
        lap_started_at: now,
        ..default()
    };
}

fn reset_player(world: &mut World, player: Entity, start: Vec3, target: Vec3) {
    let transform = Transform::from_translation(start).looking_at(target, Vec3::Y);
    *world
        .get_mut::<Transform>(player)
        .expect("player transform should exist") = transform;
    *world
        .get_mut::<LinearVelocity>(player)
        .expect("player linear velocity should exist") = LinearVelocity(Vec3::ZERO);
    *world
        .get_mut::<AngularVelocity>(player)
        .expect("player angular velocity should exist") = AngularVelocity(Vec3::ZERO);
    ground_vehicle::reset_vehicle_state(world, player);
}
