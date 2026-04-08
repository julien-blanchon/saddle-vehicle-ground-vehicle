//! Sim racing example — realistic RWD sports car with MagicFormula tires.
//!
//! Demonstrates a high-fidelity car setup for sim-racing games: stiff suspension,
//! strong aero downforce, MagicFormula tire model on all four wheels, and minimal
//! stability aids. The car rewards precision and punishes overdriving.

use bevy::prelude::*;
use ground_vehicle::GroundVehicleSurface;
use ground_vehicle_example_support as support;
use support::{spawn_overlay, spawn_surface_box, spawn_world};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle sim_racing", true);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);
    spawn_overlay(&mut commands, "ground_vehicle sim_racing");

    // Track walls — long corridor for high-speed runs
    let wall_surface = GroundVehicleSurface {
        lateral_grip_scale: 0.3,
        ..default()
    };
    for side in [-1.0_f32, 1.0] {
        for segment in 0..6 {
            let z = -40.0 * segment as f32;
            spawn_surface_box(
                &mut commands,
                &mut meshes,
                &mut materials,
                &format!(
                    "Track Wall {} {}",
                    if side < 0.0 { "L" } else { "R" },
                    segment
                ),
                Vec3::new(0.4, 0.8, 42.0),
                Transform::from_xyz(side * 8.0, 0.4, z),
                Color::srgb(0.55, 0.55, 0.58),
                wall_surface,
            );
        }
    }

    // Curbs at edges
    let curb_surface = GroundVehicleSurface {
        lateral_grip_scale: 0.65,
        ..default()
    };
    for side in [-1.0_f32, 1.0] {
        for segment in 0..12 {
            let z = -20.0 * segment as f32;
            let even = segment % 2 == 0;
            spawn_surface_box(
                &mut commands,
                &mut meshes,
                &mut materials,
                &format!("Curb {} {}", if side < 0.0 { "L" } else { "R" }, segment),
                Vec3::new(1.2, 0.08, 20.0),
                Transform::from_xyz(side * 6.8, 0.04, z),
                if even {
                    Color::srgb(0.90, 0.15, 0.12)
                } else {
                    Color::WHITE
                },
                curb_surface,
            );
        }
    }

    // Braking zone marker
    spawn_surface_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Braking Zone",
        Vec3::new(12.0, 0.02, 2.0),
        Transform::from_xyz(0.0, 0.01, -180.0),
        Color::srgb(0.85, 0.70, 0.10),
        GroundVehicleSurface::default(),
    );

    support::spawn_sim_racer_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Sim Racer",
        Transform::from_xyz(0.0, 1.0, 10.0),
        true,
    );
}
