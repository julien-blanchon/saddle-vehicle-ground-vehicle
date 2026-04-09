use avian3d::prelude::{AngularVelocity, LinearVelocity};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::ContextActivity;
use ground_vehicle::{GroundVehicleDriftTelemetry, GroundVehicleTelemetry, VehicleIntent};
use saddle_bevy_e2e::{
    action::Action,
    actions::{assertions, inspect},
    scenario::Scenario,
};

use crate::{
    ActiveVehicle, LabState,
    support::{ExampleDriver, ScriptedControlOverride},
};

#[derive(Resource, Clone, Copy)]
struct OpenWorldCrateSnapshot {
    entity: Entity,
    position: Vec3,
}

fn reset_active_vehicle(active: ActiveVehicle, transform: Transform, velocity: Vec3) -> Action {
    Action::Custom(Box::new(move |world: &mut World| {
        configure_active_vehicle(world, active, transform, velocity);
    }))
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "ground_vehicle_smoke" => Some(build_smoke()),
        "ground_vehicle_braking" => Some(build_braking()),
        "ground_vehicle_drivetrain" => Some(build_drivetrain()),
        "ground_vehicle_slope" => Some(build_slope()),
        "ground_vehicle_drift" => Some(build_drift()),
        "ground_vehicle_skid_steer" => Some(build_skid_steer()),
        "ground_vehicle_multi_axle" => Some(build_multi_axle()),
        "ground_vehicle_kart_racing" => Some(build_kart_racing()),
        "ground_vehicle_sport_bike" => Some(build_sport_bike()),
        "ground_vehicle_sim_racing" => Some(build_sim_racing()),
        "ground_vehicle_open_world" => Some(build_open_world()),
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
        "ground_vehicle_kart_racing",
        "ground_vehicle_sport_bike",
        "ground_vehicle_sim_racing",
        "ground_vehicle_open_world",
    ]
}

fn build_smoke() -> Scenario {
    Scenario::builder("ground_vehicle_smoke")
        .description("Verify the compact car settles, takes throttle, and builds forward speed.")
        .then(reset_active_vehicle(
            ActiveVehicle::Compact,
            Transform::from_xyz(0.0, 0.82, 22.0),
            Vec3::ZERO,
        ))
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
                        telemetry.speed_mps > 1.5 && telemetry.forward_speed_mps > 1.0
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
                .is_some_and(|telemetry| {
                    telemetry.speed_mps > 1.0 && telemetry.forward_speed_mps > 0.5
                })
        }))
        .then(assertions::custom(
            "compact car has ground contact",
            |world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|telemetry| telemetry.grounded_wheels >= 2 && !telemetry.airborne)
            },
        ))
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
        .description(
            "Verify the compact car can brake to a stop after building speed under throttle.",
        )
        .then(reset_active_vehicle(
            ActiveVehicle::Compact,
            Transform::from_xyz(0.0, 0.82, 46.0),
            Vec3::ZERO,
        ))
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
            set_control(
                world,
                car,
                VehicleIntent {
                    drive: 1.0,
                    ..default()
                },
            );
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
            let telemetry = world
                .get::<GroundVehicleTelemetry>(car)
                .copied()
                .expect("telemetry");
            info!(
                "[e2e] pre-brake state: speed={:.3} fwd={:.3} grounded={}",
                telemetry.speed_mps, telemetry.forward_speed_mps, telemetry.grounded_wheels,
            );
            set_control(
                world,
                car,
                VehicleIntent {
                    brake: 1.0,
                    ..default()
                },
            );
        })))
        .then(Action::WaitUntil {
            label: "compact car stopped".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|telemetry| telemetry.speed_mps < 0.3)
            }),
            max_frames: 600,
        })
        .then(assertions::custom("compact car stopped", |world| {
            let car = world.resource::<LabState>().compact;
            world
                .get::<GroundVehicleTelemetry>(car)
                .is_some_and(|telemetry| telemetry.speed_mps < 1.0)
        }))
        .then(assertions::custom(
            "compact car has ground contact after braking",
            |world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|telemetry| telemetry.grounded_wheels >= 2)
            },
        ))
        .then(assertions::custom(
            "compact car did not yaw wildly while braking",
            |world| {
                let car = world.resource::<LabState>().compact;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|telemetry| telemetry.lateral_speed_mps.abs() < 3.0)
            },
        ))
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
        .then(reset_active_vehicle(
            ActiveVehicle::Compact,
            Transform::from_xyz(0.0, 0.82, 54.0),
            Vec3::ZERO,
        ))
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
                        telemetry.selected_gear >= 2 && telemetry.engine_rpm > 1_500.0
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
        .then(reset_active_vehicle(
            ActiveVehicle::Rover,
            Transform::from_xyz(42.0, 4.7, 46.0).with_rotation(Quat::from_rotation_x(-0.28)),
            Vec3::ZERO,
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let rover = world.resource::<LabState>().rover;
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
        .description(
            "Verify the drift coupe enters a drift under drive, turn, and auxiliary brake.",
        )
        .then(reset_active_vehicle(
            ActiveVehicle::Drift,
            Transform::from_xyz(42.0, 1.18, 18.0),
            Vec3::new(0.0, 0.0, -6.0),
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let drift = world.resource::<LabState>().drift;
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
        .then(assertions::custom("drift coupe is rotating", |world| {
            let drift = world.resource::<LabState>().drift;
            let telemetry_ok = world
                .get::<GroundVehicleDriftTelemetry>(drift)
                .is_some_and(|telemetry| telemetry.drift_ratio > 0.05)
                || world
                    .get::<GroundVehicleTelemetry>(drift)
                    .is_some_and(|telemetry| telemetry.lateral_speed_mps.abs() > 0.5);
            let transform_ok = world
                .get::<Transform>(drift)
                .is_some_and(|transform| transform.rotation.to_euler(EulerRot::YXZ).0.abs() > 0.1);
            telemetry_ok || transform_ok
        }))
        .then(assertions::custom(
            "drift coupe has ground contact",
            |world| {
                let drift = world.resource::<LabState>().drift;
                world
                    .get::<GroundVehicleTelemetry>(drift)
                    .is_some_and(|telemetry| telemetry.grounded_wheels >= 2 && !telemetry.airborne)
            },
        ))
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
        .then(reset_active_vehicle(
            ActiveVehicle::Skid,
            Transform::from_xyz(0.0, 1.35, -28.0),
            Vec3::ZERO,
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let skid = world.resource::<LabState>().skid;
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
        .then(reset_active_vehicle(
            ActiveVehicle::Truck,
            Transform::from_xyz(-46.0, 1.7, 24.0),
            Vec3::ZERO,
        ))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let truck = world.resource::<LabState>().truck;
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
                let telemetry_ok = world
                    .get::<GroundVehicleTelemetry>(truck)
                    .is_some_and(|telemetry| telemetry.grounded_wheels >= 2);
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

fn build_kart_racing() -> Scenario {
    Scenario::builder("ground_vehicle_kart_racing")
        .description(
            "Drive the actual kart setup through its arcade lane: hit the boost pad, carry speed \
             into a sweeping turn toward the slippery patch, and verify the kart stays planted \
             and responsive.",
        )
        .then(reset_active_vehicle(
            ActiveVehicle::Kart,
            Transform::from_xyz(-82.0, 0.8, -18.0),
            Vec3::ZERO,
        ))
        .then(Action::WaitFrames(10))
        .then(Action::WaitUntil {
            label: "kart settled on ground".into(),
            condition: Box::new(|world| {
                let kart = world.resource::<LabState>().kart;
                world
                    .get::<GroundVehicleTelemetry>(kart)
                    .is_some_and(|t| t.grounded_wheels >= 3 && t.speed_mps < 0.5)
            }),
            max_frames: 180,
        })
        .then(Action::Custom(Box::new(|world: &mut World| {
            let kart = world.resource::<LabState>().kart;
            set_control(
                world,
                kart,
                VehicleIntent {
                    drive: 1.0,
                    ..default()
                },
            );
        })))
        .then(Action::WaitUntil {
            label: "kart reached racing speed".into(),
            condition: Box::new(|world| {
                let kart = world.resource::<LabState>().kart;
                world
                    .get::<GroundVehicleTelemetry>(kart)
                    .is_some_and(|t| t.speed_mps > 3.5)
            }),
            max_frames: 600,
        })
        .then(Action::Screenshot("kart_racing_straight".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let kart = world.resource::<LabState>().kart;
            set_control(
                world,
                kart,
                VehicleIntent {
                    drive: 1.0,
                    turn: 0.45,
                    ..default()
                },
            );
        })))
        .then(Action::WaitFrames(90))
        .then(assertions::custom(
            "kart stayed grounded through the arcade lane",
            |world| {
                let kart = world.resource::<LabState>().kart;
                world
                    .get::<GroundVehicleTelemetry>(kart)
                    .is_some_and(|t| t.grounded_wheels >= 2 && !t.airborne)
            },
        ))
        .then(assertions::custom(
            "kart maintained strong forward progress",
            |world| {
                let kart = world.resource::<LabState>().kart;
                world
                    .get::<GroundVehicleTelemetry>(kart)
                    .is_some_and(|t| t.speed_mps > 2.0 && t.forward_speed_mps > 1.0)
            },
        ))
        .then(Action::Screenshot("kart_racing_chicane_exit".into()))
        .then(Action::WaitFrames(1))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "kart_racing_telemetry",
        ))
        .then(assertions::log_summary(
            "ground_vehicle_kart_racing summary",
        ))
        .build()
}

fn build_sport_bike() -> Scenario {
    Scenario::builder("ground_vehicle_sport_bike")
        .description(
            "Use the actual sport-bike setup: accelerate through the slalom lane, hold a turn at \
             speed, and verify the bike-style configuration stays upright and composed.",
        )
        .then(reset_active_vehicle(
            ActiveVehicle::SportBike,
            Transform::from_xyz(-82.0, 1.0, 58.0),
            Vec3::ZERO,
        ))
        .then(Action::WaitFrames(10))
        .then(Action::WaitUntil {
            label: "sport bike settled on ground".into(),
            condition: Box::new(|world| {
                let bike = world.resource::<LabState>().sport_bike;
                world
                    .get::<GroundVehicleTelemetry>(bike)
                    .is_some_and(|t| t.grounded_wheels >= 3 && t.speed_mps < 0.5)
            }),
            max_frames: 180,
        })
        .then(Action::Custom(Box::new(|world: &mut World| {
            let bike = world.resource::<LabState>().sport_bike;
            set_control(
                world,
                bike,
                VehicleIntent {
                    drive: 0.9,
                    ..default()
                },
            );
        })))
        .then(Action::WaitUntil {
            label: "sport bike reached sport speed".into(),
            condition: Box::new(|world| {
                let bike = world.resource::<LabState>().sport_bike;
                world
                    .get::<GroundVehicleTelemetry>(bike)
                    .is_some_and(|t| t.speed_mps > 1.5)
            }),
            max_frames: 600,
        })
        .then(Action::Screenshot("sport_bike_straight".into()))
        .then(Action::WaitFrames(1))
        .then(Action::Custom(Box::new(|world: &mut World| {
            let bike = world.resource::<LabState>().sport_bike;
            set_control(
                world,
                bike,
                VehicleIntent {
                    drive: 0.7,
                    turn: 0.70,
                    ..default()
                },
            );
        })))
        .then(Action::WaitFrames(80))
        .then(assertions::custom(
            "sport bike stayed grounded through turn",
            |world| {
                let bike = world.resource::<LabState>().sport_bike;
                world
                    .get::<GroundVehicleTelemetry>(bike)
                    .is_some_and(|t| t.grounded_wheels >= 2 && !t.airborne)
            },
        ))
        .then(assertions::custom(
            "sport bike roll angle reasonable (no tip-over)",
            |world| {
                let bike = world.resource::<LabState>().sport_bike;
                world.get::<Transform>(bike).is_some_and(|transform| {
                    let (_, _, roll) = transform.rotation.to_euler(EulerRot::YXZ);
                    roll.abs() < 1.0
                })
            },
        ))
        .then(Action::Screenshot("sport_bike_turn".into()))
        .then(Action::WaitFrames(1))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "sport_bike_telemetry",
        ))
        .then(assertions::log_summary("ground_vehicle_sport_bike summary"))
        .build()
}

fn build_sim_racing() -> Scenario {
    Scenario::builder("ground_vehicle_sim_racing")
        .description(
            "Use the actual sim-racing setup: launch down the corridor under sustained throttle, \
             verify it upshifts cleanly, and confirm the high-grip race car stays composed.",
        )
        .then(reset_active_vehicle(
            ActiveVehicle::SimRacer,
            Transform::from_xyz(82.0, 1.0, 22.0),
            Vec3::ZERO,
        ))
        .then(Action::WaitFrames(10))
        .then(Action::WaitUntil {
            label: "sim racer settled on grid".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().sim_racer;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|t| t.grounded_wheels >= 4 && t.speed_mps < 0.3)
            }),
            max_frames: 180,
        })
        .then(Action::Custom(Box::new(|world: &mut World| {
            let car = world.resource::<LabState>().sim_racer;
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
        .then(Action::Screenshot("sim_racing_launch".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "sim racer upshifted at least once".into(),
            condition: Box::new(|world| {
                let car = world.resource::<LabState>().sim_racer;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|t| t.selected_gear >= 2)
            }),
            max_frames: 600,
        })
        .then(assertions::custom(
            "engine RPM in operating band during sim racing",
            |world| {
                let car = world.resource::<LabState>().sim_racer;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|t| t.engine_rpm > 800.0 && t.engine_rpm < 8_000.0)
            },
        ))
        .then(assertions::custom(
            "vehicle did not go airborne during lap",
            |world| {
                let car = world.resource::<LabState>().sim_racer;
                world
                    .get::<GroundVehicleTelemetry>(car)
                    .is_some_and(|t| !t.airborne && t.grounded_wheels >= 2)
            },
        ))
        .then(assertions::custom(
            "drift ratio stayed low (sim grip mode)",
            |world| {
                let car = world.resource::<LabState>().sim_racer;
                world
                    .get::<GroundVehicleDriftTelemetry>(car)
                    .is_some_and(|drift| drift.drift_ratio < 0.35)
            },
        ))
        .then(Action::Screenshot("sim_racing_cruising".into()))
        .then(Action::WaitFrames(1))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "sim_racing_telemetry",
        ))
        .then(assertions::log_summary("ground_vehicle_sim_racing summary"))
        .build()
}

fn build_open_world() -> Scenario {
    Scenario::builder("ground_vehicle_open_world")
        .description(
            "Drive the open-world sedan through the obstacle lane: hit the loose crates, keep \
             building speed, and verify the forgiving sedan remains stable after contact.",
        )
        .then(reset_active_vehicle(
            ActiveVehicle::Sedan,
            Transform::from_xyz(87.0, 1.2, -42.0),
            Vec3::ZERO,
        ))
        .then(Action::WaitFrames(10))
        .then(Action::WaitUntil {
            label: "sedan settled on the open-world lane".into(),
            condition: Box::new(|world| {
                let sedan = world.resource::<LabState>().sedan;
                world
                    .get::<GroundVehicleTelemetry>(sedan)
                    .is_some_and(|t| t.grounded_wheels >= 4 && t.speed_mps < 0.3)
            }),
            max_frames: 240,
        })
        .then(Action::Custom(Box::new(|world: &mut World| {
            let crate_entity =
                named_entity(world, "Lab Sedan Crate 1").expect("lab sedan crate should exist");
            let position = world
                .get::<Transform>(crate_entity)
                .map(|transform| transform.translation)
                .expect("lab sedan crate transform should exist");
            world.insert_resource(OpenWorldCrateSnapshot {
                entity: crate_entity,
                position,
            });

            let sedan = world.resource::<LabState>().sedan;
            set_control(
                world,
                sedan,
                VehicleIntent {
                    drive: 1.0,
                    ..default()
                },
            );
        })))
        .then(Action::Screenshot("open_world_launch".into()))
        .then(Action::WaitFrames(1))
        .then(Action::WaitUntil {
            label: "sedan struck obstacle crate".into(),
            condition: Box::new(|world| {
                let snapshot = world.resource::<OpenWorldCrateSnapshot>();
                world
                    .get::<Transform>(snapshot.entity)
                    .is_some_and(|transform| {
                        transform.translation.distance(snapshot.position) > 0.4
                    })
            }),
            max_frames: 240,
        })
        .then(assertions::custom(
            "sedan moved the obstacle crate",
            |world| {
                let snapshot = world.resource::<OpenWorldCrateSnapshot>();
                world
                    .get::<Transform>(snapshot.entity)
                    .is_some_and(|transform| {
                        transform.translation.distance(snapshot.position) > 0.4
                    })
            },
        ))
        .then(assertions::custom(
            "sedan stayed driveable after contact",
            |world| {
                let sedan = world.resource::<LabState>().sedan;
                world
                    .get::<GroundVehicleTelemetry>(sedan)
                    .is_some_and(|telemetry| {
                        telemetry.speed_mps > 1.0 && telemetry.grounded_wheels >= 2
                    })
            },
        ))
        .then(Action::Screenshot("open_world_obstacle_lane".into()))
        .then(Action::WaitFrames(1))
        .then(inspect::log_component::<GroundVehicleTelemetry>(
            "open_world_telemetry",
        ))
        .then(assertions::log_summary("ground_vehicle_open_world summary"))
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
        state.sport_bike,
        state.sim_racer,
        state.kart,
        state.sedan,
    ] {
        world
            .entity_mut(entity)
            .insert(ContextActivity::<ExampleDriver>::INACTIVE);
    }

    let entity = active_vehicle_entity(state, active);
    world
        .entity_mut(entity)
        .insert(ContextActivity::<ExampleDriver>::ACTIVE);
}

fn configure_active_vehicle(
    world: &mut World,
    active: ActiveVehicle,
    transform: Transform,
    velocity: Vec3,
) {
    set_active_vehicle(world, active);
    let state = *world.resource::<LabState>();
    let entity = active_vehicle_entity(state, active);
    reset_vehicle(world, entity, transform, velocity);
}

fn active_vehicle_entity(state: LabState, active: ActiveVehicle) -> Entity {
    match active {
        ActiveVehicle::Compact => state.compact,
        ActiveVehicle::Drift => state.drift,
        ActiveVehicle::Truck => state.truck,
        ActiveVehicle::Skid => state.skid,
        ActiveVehicle::Rover => state.rover,
        ActiveVehicle::SportBike => state.sport_bike,
        ActiveVehicle::SimRacer => state.sim_racer,
        ActiveVehicle::Kart => state.kart,
        ActiveVehicle::Sedan => state.sedan,
    }
}

fn named_entity(world: &mut World, name: &str) -> Option<Entity> {
    let mut query = world.query::<(Entity, &Name)>();
    query
        .iter(world)
        .find_map(|(entity, candidate)| (candidate.as_str() == name).then_some(entity))
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
    // Reset wheel & powertrain state immediately so the suspension damper
    // doesn't spike from stale previous_suspension_length after teleport.
    ground_vehicle::reset_vehicle_state(world, entity);
    world
        .entity_mut(entity)
        .insert(ScriptedControlOverride(None));
    *world
        .get_mut::<VehicleIntent>(entity)
        .expect("vehicle intent should exist") = VehicleIntent::default();
}
