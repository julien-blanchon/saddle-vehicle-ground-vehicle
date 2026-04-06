use avian3d::prelude::{AngularVelocity, LinearVelocity};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::ContextActivity;
use ground_vehicle::{
    GroundVehicleDriftTelemetry, GroundVehicleReset, GroundVehicleTelemetry, VehicleIntent,
};
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};

use crate::{
    ActiveVehicle, LabState,
    support::{ExampleDriver, ScriptedControlOverride},
};

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "ground_vehicle_smoke" => Some(build_smoke()),
        "ground_vehicle_braking" => Some(build_braking()),
        "ground_vehicle_drivetrain" => Some(build_drivetrain()),
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
        "ground_vehicle_drivetrain",
        "ground_vehicle_slope",
        "ground_vehicle_drift",
        "ground_vehicle_skid_steer",
        "ground_vehicle_multi_axle",
    ]
}

fn build_smoke() -> Scenario {
    Scenario::builder("ground_vehicle_smoke")
        .description(
            "Verify the compact car settles, takes throttle, and builds forward speed.",
        )
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_active_vehicle(world, ActiveVehicle::Compact);
            let car = world.resource::<LabState>().compact;
            reset_vehicle(world, car, Transform::from_xyz(0.0, 0.82, 22.0), Vec3::ZERO);
        })))
        .then(Action::WaitFrames(10))
        .then(Action::WaitUntil {
            label: "compact car settled on ground".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|t| t.grounded_wheels >= 3 && t.speed_mps < 0.5)
            }),
            max_frames: 240,
        })
        .then(Action::Custom(Box::new(|world: &mut World| {
            let car = world.resource::<LabState>().compact;
            let telemetry = world
                .get::<GroundVehicleTelemetry>(car)
                .copied()
                .expect("telemetry exists");
            info!(
                "[e2e] pre-throttle state: gear={} rpm={:.0} speed={:.3} grounded={}",
                telemetry.selected_gear,
                telemetry.engine_rpm,
                telemetry.speed_mps,
                telemetry.grounded_wheels,
            );
            set_control(
                world,
                car,
                VehicleIntent {
                    drive: 1.0,
                    ..default()
                },
            );
        })))
        .then(Action::Screenshot("ground_vehicle_smoke_start".into()))
        // Wait for the car to build speed (give plenty of time for physics to settle)
        .then(Action::WaitUntil {
            label: "compact car reached speed".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|telemetry| {
                        telemetry.speed_mps > 1.5
                            && telemetry.forward_speed_mps > 1.0
                    })
            }),
            max_frames: 600,
        })
        .then(Action::Custom(Box::new(|world: &mut World| {
            let car = world.resource::<LabState>().compact;
            let telemetry = world
                .get::<GroundVehicleTelemetry>(car)
                .copied()
                .expect("telemetry exists");
            info!(
                "[e2e] throttle result: gear={} rpm={:.0} speed={:.3} fwd={:.3} grounded={}",
                telemetry.selected_gear,
                telemetry.engine_rpm,
                telemetry.speed_mps,
                telemetry.forward_speed_mps,
                telemetry.grounded_wheels,
            );
        })))
        .then(assertions::custom("compact car built speed", |world| {
            let car = world.resource::<LabState>().compact;
            world
                .get::<GroundVehicleTelemetry>(car)
                .is_some_and(|telemetry| telemetry.speed_mps > 1.0 && telemetry.forward_speed_mps > 0.5)
        }))
        .then(assertions::custom("compact car has ground contact", |world| {
            let car = world.resource::<LabState>().compact;
            world
                .get::<GroundVehicleTelemetry>(car)
                .is_some_and(|telemetry| telemetry.grounded_wheels >= 2 && !telemetry.airborne)
        }))
        .then(assertions::custom(
            "compact car launch stayed out of drift",
            |world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleDriftTelemetry>(car)
                    .is_some_and(|drift| !drift.drifting && drift.drift_ratio < 0.3)
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
        .description("Verify the compact car can brake to a stop after building speed under throttle.")
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_active_vehicle(world, ActiveVehicle::Compact);
            let car = world.resource::<LabState>().compact;
            reset_vehicle(world, car, Transform::from_xyz(0.0, 0.82, 46.0), Vec3::ZERO);
        })))
        .then(Action::WaitFrames(10))
        .then(Action::WaitUntil {
            label: "compact car settled for braking".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|t| t.grounded_wheels >= 3 && t.speed_mps < 0.5)
            }),
            max_frames: 240,
        })
        // Phase 1: Build speed with throttle
        .then(Action::Custom(Box::new(|world: &mut World| {
            let car = world.resource::<LabState>().compact;
            set_control(world, car, VehicleIntent { drive: 1.0, ..default() });
        })))
        .then(Action::WaitUntil {
            label: "compact car built some speed".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|t| t.speed_mps > 1.0)
            }),
            max_frames: 600,
        })
        .then(Action::Screenshot("ground_vehicle_braking_entry".into()))
        // Phase 2: Full brake
        .then(Action::Custom(Box::new(|world: &mut World| {
            let car = world.resource::<LabState>().compact;
            let telemetry = world.get::<GroundVehicleTelemetry>(car).copied().expect("telemetry");
            info!(
                "[e2e] pre-brake state: speed={:.3} fwd={:.3} grounded={}",
                telemetry.speed_mps, telemetry.forward_speed_mps, telemetry.grounded_wheels,
            );
            set_control(world, car, VehicleIntent { brake: 1.0, ..default() });
        })))
        .then(Action::WaitUntil {
            label: "compact car stopped".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().compact;
                world.get::<GroundVehicleTelemetry>(car).is_some_and(|telemetry| {
                    telemetry.speed_mps < 0.3
                })
            }),
            max_frames: 600,
        })
        .then(assertions::custom("compact car stopped", |world| {
            let car = world.resource::<LabState>().compact;
            world
                .get::<GroundVehicleTelemetry>(car)
                .is_some_and(|telemetry| telemetry.speed_mps < 1.0)
        }))
        .then(assertions::custom("compact car has ground contact after braking", |world| {
            let car = world.resource::<LabState>().compact;
            world.get::<GroundVehicleTelemetry>(car).is_some_and(|telemetry| {
                telemetry.grounded_wheels >= 2
            })
        }))
        .then(assertions::custom("compact car did not yaw wildly while braking", |world| {
            let car = world.resource::<LabState>().compact;
            world
                .get::<GroundVehicleTelemetry>(car)
                .is_some_and(|telemetry| telemetry.lateral_speed_mps.abs() < 3.0)
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

fn build_drivetrain() -> Scenario {
    Scenario::builder("ground_vehicle_drivetrain")
        .description(
            "Verify the compact car upshifts under load and reports engine RPM through telemetry.",
        )
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_active_vehicle(world, ActiveVehicle::Compact);
            let car = world.resource::<LabState>().compact;
            reset_vehicle(world, car, Transform::from_xyz(0.0, 0.82, 54.0), Vec3::ZERO);
        })))
        .then(Action::WaitFrames(10))
        .then(Action::WaitUntil {
            label: "compact car settled for drivetrain".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|t| t.grounded_wheels >= 4 && t.speed_mps < 0.3)
            }),
            max_frames: 180,
        })
        .then(Action::Custom(Box::new(|world: &mut World| {
            let car = world.resource::<LabState>().compact;
            set_control(
                world,
                car,
                VehicleIntent {
                    drive: 1.0,
                    ..default()
                },
            );
        })))
        .then(Action::WaitFrames(15))
        .then(Action::Screenshot(
            "ground_vehicle_drivetrain_launch".into(),
        ))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "compact car upshifted".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|telemetry| {
                        telemetry.selected_gear >= 2
                            && telemetry.engine_rpm > 1_500.0
                    })
            }),
            max_frames: 600,
        })
        .then(assertions::custom(
            "compact car reported higher gear",
            |world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|telemetry| telemetry.selected_gear >= 2)
            },
        ))
        .then(assertions::custom(
            "compact car reported engine rpm",
            |world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|telemetry| {
                        telemetry.engine_rpm > 1_000.0 && telemetry.engine_rpm < 7_500.0
                    })
            },
        ))
        .then(Action::Screenshot(
            "ground_vehicle_drivetrain_shifted".into(),
        ))
        .then(Action::WaitFrames(1))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "ground_vehicle_drivetrain_telemetry",
        ))
        .then(assertions::log_summary("ground_vehicle_drivetrain summary"))
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
                VehicleIntent {
                    drive: 0.0,
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
        .description("Verify the drift coupe enters a drift under drive, turn, and auxiliary brake.")
        .then(Action::Custom(Box::new(|world: &mut World| {
            set_active_vehicle(world, ActiveVehicle::Drift);
            let drift = world.resource::<LabState>().drift;
            // Moderate initial velocity — the physics sim limits effective speed
            reset_vehicle(
                world,
                drift,
                Transform::from_xyz(42.0, 1.18, 18.0),
                Vec3::new(0.0, 0.0, -6.0),
            );
            set_control(
                world,
                drift,
                VehicleIntent {
                    drive: 1.0,
                    turn: 0.72,
                    auxiliary_brake: 1.0,
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
                    .get::<GroundVehicleDriftTelemetry>(drift)
                    .is_some_and(|telemetry| telemetry.drifting || telemetry.drift_ratio > 0.10)
            }),
            max_frames: 300,
        })
        .then(assertions::custom(
            "drift coupe is rotating",
            |world| {
                let drift = world.resource::<LabState>().drift;
                let telemetry_ok =
                    world
                        .get::<GroundVehicleDriftTelemetry>(drift)
                        .is_some_and(|telemetry| telemetry.drift_ratio > 0.05)
                    || world
                        .get::<GroundVehicleTelemetry>(drift)
                        .is_some_and(|telemetry| telemetry.lateral_speed_mps.abs() > 0.5);
                let transform_ok = world.get::<Transform>(drift).is_some_and(|transform| {
                    transform.rotation.to_euler(EulerRot::YXZ).0.abs() > 0.1
                });
                telemetry_ok || transform_ok
            },
        ))
        .then(assertions::custom("drift coupe has ground contact", |world| {
            let drift = world.resource::<LabState>().drift;
            world
                .get::<GroundVehicleTelemetry>(drift)
                .is_some_and(|telemetry| telemetry.grounded_wheels >= 2 && !telemetry.airborne)
        }))
        .then(assertions::custom(
            "drift coupe showed lateral movement",
            |world| {
                let drift = world.resource::<LabState>().drift;
                world
                    .get::<GroundVehicleDriftTelemetry>(drift)
                    .is_some_and(|telemetry| telemetry.drift_ratio > 0.05)
                    || world
                        .get::<GroundVehicleTelemetry>(drift)
                        .is_some_and(|telemetry| telemetry.lateral_speed_mps.abs() > 0.3)
            },
        ))
        .then(Action::Screenshot("ground_vehicle_drift_state".into()))
        .then(Action::WaitFrames(1))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "ground_vehicle_drift_telemetry",
        ))
        .then(inspect::log_component::<GroundVehicleDriftTelemetry>(
            "ground_vehicle_drift_helper",
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
                VehicleIntent {
                    drive: 0.15,
                    turn: 1.0,
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
                telemetry.forward_speed_mps.abs() < 6.0
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
                VehicleIntent {
                    drive: 0.9,
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
                            telemetry.grounded_wheels >= 2
                        });
                let roll_ok = world.get::<Transform>(truck).is_some_and(|transform| {
                    let (_, _, roll) = transform.rotation.to_euler(EulerRot::YXZ);
                    roll.abs() < 1.2
                });
                telemetry_ok && roll_ok
            },
        ))
        .then(assertions::custom(
            "truck kept wheels on the ground",
            |world| {
                let truck = world.resource::<LabState>().truck;
                world
                    .get::<GroundVehicleTelemetry>(truck)
                    .is_some_and(|telemetry| telemetry.grounded_wheels >= 3 && !telemetry.airborne)
            },
        ))
        .then(assertions::custom(
            "truck stayed out of a drift state",
            |world| {
                let truck = world.resource::<LabState>().truck;
                world
                    .get::<GroundVehicleDriftTelemetry>(truck)
                    .is_some_and(|drift| !drift.drifting && drift.drift_ratio < 0.2)
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

fn set_control(world: &mut World, entity: Entity, control: VehicleIntent) {
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
        .insert((ScriptedControlOverride(None), GroundVehicleReset));
    *world
        .get_mut::<VehicleIntent>(entity)
        .expect("vehicle intent should exist") = VehicleIntent::default();
}
