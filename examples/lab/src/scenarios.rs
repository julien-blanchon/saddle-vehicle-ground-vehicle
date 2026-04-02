use avian3d::prelude::{AngularVelocity, LinearVelocity};
use bevy::prelude::*;
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};
use bevy_enhanced_input::prelude::ContextActivity;
use ground_vehicle::{GroundVehicleControl, GroundVehicleTelemetry};

use crate::{
    ActiveVehicle, LabState,
    support::{ExampleDriver, ScriptedControlOverride},
};

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "ground_vehicle_smoke" => Some(build_smoke()),
        "ground_vehicle_braking" => Some(build_braking()),
        "ground_vehicle_slope" => Some(build_slope()),
        "ground_vehicle_drift" => Some(build_drift()),
        "ground_vehicle_skid_steer" => Some(build_skid_steer()),
        "ground_vehicle_multi_axle" => Some(build_multi_axle()),
        _ => None,
    }
}

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "ground_vehicle_smoke",
        "ground_vehicle_braking",
        "ground_vehicle_slope",
        "ground_vehicle_drift",
        "ground_vehicle_skid_steer",
        "ground_vehicle_multi_axle",
    ]
}

fn build_smoke() -> Scenario {
    Scenario::builder("ground_vehicle_smoke")
        .description(
            "Verify the compact car settles, takes throttle, and reaches a useful forward speed.",
        )
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_active_vehicle(world, ActiveVehicle::Compact);
            let car = world.resource::<LabState>().compact;
            reset_vehicle(world, car, Transform::from_xyz(0.0, 1.25, 22.0), Vec3::ZERO);
            set_control(
                world,
                car,
                GroundVehicleControl {
                    throttle: 1.0,
                    ..default()
                },
            );
        })))
        .then(Action::WaitFrames(45))
        .then(Action::Screenshot("ground_vehicle_smoke_start".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "compact car reached speed".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|telemetry| {
                        telemetry.speed_mps > 10.0
                            && telemetry.forward_speed_mps > 8.0
                            && telemetry.grounded_wheels >= 3
                    })
            }),
            max_frames: 240,
        })
        .then(assertions::custom("compact car built speed", |world| {
            let car = world.resource::<LabState>().compact;
            world
                .get::<GroundVehicleTelemetry>(car)
                .is_some_and(|telemetry| telemetry.speed_mps > 10.0)
        }))
        .then(assertions::custom("compact car stayed planted", |world| {
            let car = world.resource::<LabState>().compact;
            world
                .get::<GroundVehicleTelemetry>(car)
                .is_some_and(|telemetry| telemetry.grounded_wheels >= 4 && !telemetry.airborne)
        }))
        .then(assertions::custom(
            "compact car launch stayed out of drift",
            |world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|telemetry| !telemetry.drifting && telemetry.drift_ratio < 0.2)
            },
        ))
        .then(Action::Screenshot("ground_vehicle_smoke_speed".into()))
        .then(Action::WaitFrames(1))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "ground_vehicle_smoke_telemetry",
        ))
        .then(assertions::log_summary("ground_vehicle_smoke summary"))
        .build()
}

fn build_braking() -> Scenario {
    Scenario::builder("ground_vehicle_braking")
        .description("Verify the compact car can brake down from speed without overshooting the expected zone.")
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_active_vehicle(world, ActiveVehicle::Compact);
            let car = world.resource::<LabState>().compact;
            reset_vehicle(
                world,
                car,
                Transform::from_xyz(0.0, 1.25, 46.0),
                Vec3::new(0.0, 0.0, -14.0),
            );
            set_control(
                world,
                car,
                GroundVehicleControl {
                    brake: 1.0,
                    ..default()
                },
            );
        })))
        .then(Action::WaitFrames(30))
        .then(Action::Screenshot("ground_vehicle_braking_entry".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "compact car stopped".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().compact;
                world.get::<GroundVehicleTelemetry>(car).is_some_and(|telemetry| {
                    telemetry.speed_mps < 1.0 && telemetry.grounded_wheels >= 3
                })
            }),
            max_frames: 320,
        })
        .then(assertions::custom("compact car stopped in range", |world| {
            let car = world.resource::<LabState>().compact;
            let stopped = world
                .get::<GroundVehicleTelemetry>(car)
                .is_some_and(|telemetry| telemetry.speed_mps < 1.0);
            let distance_ok = world
                .get::<Transform>(car)
                .is_some_and(|transform| transform.translation.z > 18.0);
            stopped && distance_ok
        }))
        .then(assertions::custom("compact car kept contact under braking", |world| {
            let car = world.resource::<LabState>().compact;
            world.get::<GroundVehicleTelemetry>(car).is_some_and(|telemetry| {
                telemetry.grounded_wheels >= 4 && !telemetry.airborne
            })
        }))
        .then(assertions::custom("compact car did not yaw wildly while braking", |world| {
            let car = world.resource::<LabState>().compact;
            world
                .get::<GroundVehicleTelemetry>(car)
                .is_some_and(|telemetry| telemetry.lateral_speed_mps.abs() < 1.0)
        }))
        .then(Action::Screenshot("ground_vehicle_braking_stop".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let car = world.resource::<LabState>().compact;
            let telemetry = world
                .get::<GroundVehicleTelemetry>(car)
                .copied()
                .expect("compact car telemetry should exist");
            let translation = world
                .get::<Transform>(car)
                .map(|transform| transform.translation)
                .expect("compact car transform should exist");
            info!(
                "[e2e] braking end state: speed={:.3} forward={:.3} grounded={} z={:.3}",
                telemetry.speed_mps,
                telemetry.forward_speed_mps,
                telemetry.grounded_wheels,
                translation.z,
            );
        })))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "ground_vehicle_braking_telemetry",
        ))
        .then(assertions::log_summary("ground_vehicle_braking summary"))
        .build()
}

fn build_slope() -> Scenario {
    Scenario::builder("ground_vehicle_slope")
        .description("Verify the slope rover holds position on the ramp under brake without jittery sliding.")
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_active_vehicle(world, ActiveVehicle::Rover);
            let rover = world.resource::<LabState>().rover;
            reset_vehicle(
                world,
                rover,
                Transform::from_xyz(42.0, 4.7, 46.0).with_rotation(Quat::from_rotation_x(-0.28)),
                Vec3::ZERO,
            );
            set_control(
                world,
                rover,
                GroundVehicleControl {
                    brake: 1.0,
                    ..default()
                },
            );
        })))
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("ground_vehicle_slope_hold".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(180))
        .then(assertions::custom("slope rover stayed near ramp start", |world| {
            let rover = world.resource::<LabState>().rover;
            let transform_ok = world.get::<Transform>(rover).is_some_and(|transform| {
                transform.translation.distance(Vec3::new(42.0, 4.7, 46.0)) < 2.0
            });
            let telemetry_ok = world
                .get::<GroundVehicleTelemetry>(rover)
                .is_some_and(|telemetry| telemetry.speed_mps < 1.0);
            transform_ok && telemetry_ok
        }))
        .then(assertions::custom("slope rover stayed grounded on the ramp", |world| {
            let rover = world.resource::<LabState>().rover;
            world.get::<GroundVehicleTelemetry>(rover).is_some_and(|telemetry| {
                telemetry.grounded_wheels >= 4 && !telemetry.airborne
            })
        }))
        .then(assertions::custom("slope rover aligned to a sloped surface normal", |world| {
            let rover = world.resource::<LabState>().rover;
            world.get::<GroundVehicleTelemetry>(rover).is_some_and(|telemetry| {
                telemetry.average_ground_normal.y < 0.99
                    && telemetry.average_ground_normal.z.abs() > 0.1
            })
        }))
        .then(Action::Screenshot("ground_vehicle_slope_settled".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let rover = world.resource::<LabState>().rover;
            let telemetry = world
                .get::<GroundVehicleTelemetry>(rover)
                .copied()
                .expect("slope rover telemetry should exist");
            let translation = world
                .get::<Transform>(rover)
                .map(|transform| transform.translation)
                .expect("slope rover transform should exist");
            info!(
                "[e2e] slope end state: speed={:.3} forward={:.3} grounded={} pos=({:.3}, {:.3}, {:.3})",
                telemetry.speed_mps,
                telemetry.forward_speed_mps,
                telemetry.grounded_wheels,
                translation.x,
                translation.y,
                translation.z,
            );
        })))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "ground_vehicle_slope_telemetry",
        ))
        .then(assertions::log_summary("ground_vehicle_slope summary"))
        .build()
}

fn build_drift() -> Scenario {
    Scenario::builder("ground_vehicle_drift")
        .description("Verify the drift coupe enters a drift under throttle, steer, and handbrake.")
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_active_vehicle(world, ActiveVehicle::Drift);
            let drift = world.resource::<LabState>().drift;
            reset_vehicle(
                world,
                drift,
                Transform::from_xyz(42.0, 1.18, 18.0),
                Vec3::new(0.0, 0.0, -16.0),
            );
            set_control(
                world,
                drift,
                GroundVehicleControl {
                    throttle: 1.0,
                    steering: 0.72,
                    handbrake: 1.0,
                    ..default()
                },
            );
        })))
        .then(Action::WaitFrames(45))
        .then(Action::Screenshot("ground_vehicle_drift_entry".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "drift telemetry triggered".into(),
            condition: Box::new(|world| {
                let drift = world.resource::<LabState>().drift;
                world
                    .get::<GroundVehicleTelemetry>(drift)
                    .is_some_and(|telemetry| telemetry.drifting && telemetry.drift_ratio > 0.18)
            }),
            max_frames: 200,
        })
        .then(assertions::custom(
            "drift coupe is rotating in drift",
            |world| {
                let drift = world.resource::<LabState>().drift;
                let telemetry_ok =
                    world
                        .get::<GroundVehicleTelemetry>(drift)
                        .is_some_and(|telemetry| {
                            telemetry.drifting && telemetry.lateral_speed_mps.abs() > 2.0
                        });
                let transform_ok = world.get::<Transform>(drift).is_some_and(|transform| {
                    transform.rotation.to_euler(EulerRot::YXZ).0.abs() > 0.2
                });
                telemetry_ok && transform_ok
            },
        ))
        .then(assertions::custom("drift coupe stayed planted", |world| {
            let drift = world.resource::<LabState>().drift;
            world
                .get::<GroundVehicleTelemetry>(drift)
                .is_some_and(|telemetry| telemetry.grounded_wheels >= 3 && !telemetry.airborne)
        }))
        .then(assertions::custom(
            "drift coupe built strong lateral speed",
            |world| {
                let drift = world.resource::<LabState>().drift;
                world
                    .get::<GroundVehicleTelemetry>(drift)
                    .is_some_and(|telemetry| {
                        telemetry.lateral_speed_mps.abs() > 5.0 && telemetry.drift_ratio > 0.5
                    })
            },
        ))
        .then(Action::Screenshot("ground_vehicle_drift_state".into()))
        .then(Action::WaitFrames(1))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "ground_vehicle_drift_telemetry",
        ))
        .then(assertions::log_summary("ground_vehicle_drift summary"))
        .build()
}

fn build_skid_steer() -> Scenario {
    Scenario::builder("ground_vehicle_skid_steer")
        .description("Verify the skid vehicle turns through left/right drive split instead of wheel steer angles.")
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_active_vehicle(world, ActiveVehicle::Skid);
            let skid = world.resource::<LabState>().skid;
            reset_vehicle(
                world,
                skid,
                Transform::from_xyz(0.0, 1.35, -28.0),
                Vec3::ZERO,
            );
            set_control(
                world,
                skid,
                GroundVehicleControl {
                    throttle: 0.15,
                    steering: 1.0,
                    ..default()
                },
            );
        })))
        .then(Action::WaitFrames(45))
        .then(Action::Screenshot("ground_vehicle_skid_steer_entry".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "skid vehicle yaw changed".into(),
            condition: Box::new(|world| {
                let skid = world.resource::<LabState>().skid;
                world.get::<Transform>(skid).is_some_and(|transform| {
                    transform.rotation.to_euler(EulerRot::YXZ).0.abs() > 0.35
                })
            }),
            max_frames: 220,
        })
        .then(assertions::custom("skid vehicle yawed without losing support", |world| {
            let skid = world.resource::<LabState>().skid;
            let yawed = world.get::<Transform>(skid).is_some_and(|transform| {
                transform.rotation.to_euler(EulerRot::YXZ).0.abs() > 0.35
            });
            let telemetry_ok = world.get::<GroundVehicleTelemetry>(skid).is_some_and(|telemetry| {
                telemetry.grounded_wheels >= 4 && !telemetry.airborne
            });
            yawed && telemetry_ok
        }))
        .then(assertions::custom("skid vehicle turn stayed mostly differential", |world| {
            let skid = world.resource::<LabState>().skid;
            world.get::<GroundVehicleTelemetry>(skid).is_some_and(|telemetry| {
                telemetry.forward_speed_mps.abs() < 4.0 && telemetry.speed_mps > 1.0
            })
        }))
        .then(assertions::custom(
            "skid vehicle kept near-zero wheel steer angle",
            |world| {
                let skid = world.resource::<LabState>().skid;
                world
                    .get::<GroundVehicleTelemetry>(skid)
                    .is_some_and(|telemetry| telemetry.average_steer_angle_rad.abs() < 0.05)
            },
        ))
        .then(Action::Screenshot("ground_vehicle_skid_steer_turn".into()))
        .then(Action::WaitFrames(1))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "ground_vehicle_skid_steer_telemetry",
        ))
        .then(assertions::log_summary("ground_vehicle_skid_steer summary"))
        .build()
}

fn build_multi_axle() -> Scenario {
    Scenario::builder("ground_vehicle_multi_axle")
        .description("Verify the cargo truck remains stable while crossing the bump course.")
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_active_vehicle(world, ActiveVehicle::Truck);
            let truck = world.resource::<LabState>().truck;
            reset_vehicle(
                world,
                truck,
                Transform::from_xyz(-46.0, 1.7, 24.0),
                Vec3::ZERO,
            );
            set_control(
                world,
                truck,
                GroundVehicleControl {
                    throttle: 0.9,
                    ..default()
                },
            );
        })))
        .then(Action::WaitFrames(60))
        .then(Action::Screenshot("ground_vehicle_multi_axle_entry".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitFrames(220))
        .then(assertions::custom(
            "truck stayed upright and kept support",
            |world| {
                let truck = world.resource::<LabState>().truck;
                let telemetry_ok =
                    world
                        .get::<GroundVehicleTelemetry>(truck)
                        .is_some_and(|telemetry| {
                            telemetry.grounded_wheels >= 3 && telemetry.speed_mps > 4.0
                        });
                let roll_ok = world.get::<Transform>(truck).is_some_and(|transform| {
                    let (_, _, roll) = transform.rotation.to_euler(EulerRot::YXZ);
                    roll.abs() < 0.9
                });
                telemetry_ok && roll_ok
            },
        ))
        .then(assertions::custom(
            "truck kept most wheels on the ground",
            |world| {
                let truck = world.resource::<LabState>().truck;
                world
                    .get::<GroundVehicleTelemetry>(truck)
                    .is_some_and(|telemetry| telemetry.grounded_wheels >= 5 && !telemetry.airborne)
            },
        ))
        .then(assertions::custom(
            "truck stayed out of a drift state",
            |world| {
                let truck = world.resource::<LabState>().truck;
                world
                    .get::<GroundVehicleTelemetry>(truck)
                    .is_some_and(|telemetry| !telemetry.drifting && telemetry.drift_ratio < 0.2)
            },
        ))
        .then(Action::Screenshot(
            "ground_vehicle_multi_axle_midcourse".into(),
        ))
        .then(Action::WaitFrames(1))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "ground_vehicle_multi_axle_telemetry",
        ))
        .then(assertions::log_summary("ground_vehicle_multi_axle summary"))
        .build()
}

fn set_active_vehicle(world: &mut World, active: ActiveVehicle) {
    let state = *world.resource::<LabState>();
    world.resource_mut::<LabState>().active = active;

    for entity in [
        state.compact,
        state.drift,
        state.truck,
        state.skid,
        state.rover,
    ] {
        world
            .entity_mut(entity)
            .insert(ContextActivity::<ExampleDriver>::INACTIVE);
    }

    let entity = match active {
        ActiveVehicle::Compact => state.compact,
        ActiveVehicle::Drift => state.drift,
        ActiveVehicle::Truck => state.truck,
        ActiveVehicle::Skid => state.skid,
        ActiveVehicle::Rover => state.rover,
    };
    world
        .entity_mut(entity)
        .insert(ContextActivity::<ExampleDriver>::ACTIVE);
}

fn set_control(world: &mut World, entity: Entity, control: GroundVehicleControl) {
    world
        .entity_mut(entity)
        .insert(ScriptedControlOverride(Some(control)));
}

fn reset_vehicle(world: &mut World, entity: Entity, transform: Transform, velocity: Vec3) {
    *world
        .get_mut::<Transform>(entity)
        .expect("vehicle transform should exist") = transform;
    *world
        .get_mut::<LinearVelocity>(entity)
        .expect("vehicle linear velocity should exist") = LinearVelocity(velocity);
    *world
        .get_mut::<AngularVelocity>(entity)
        .expect("vehicle angular velocity should exist") = AngularVelocity(Vec3::ZERO);
    world
        .entity_mut(entity)
        .insert(ScriptedControlOverride(None));
    *world
        .get_mut::<GroundVehicleControl>(entity)
        .expect("vehicle control should exist") = GroundVehicleControl::default();
}
