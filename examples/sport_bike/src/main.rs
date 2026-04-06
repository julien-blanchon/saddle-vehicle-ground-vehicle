//! Sport bike example — narrow, agile motorcycle-feel vehicle.
//!
//! Demonstrates a lightweight bike-like vehicle with very narrow track width,
//! high-revving engine, quick steering, and strong upright stabilization.
//! Perfect starting point for motorcycle, scooter, or two-wheel racing games.

use bevy::prelude::*;
use ground_vehicle::GroundVehicleSurface;
use ground_vehicle_example_support as support;
use support::{GroundVehicleExamplePane, spawn_overlay, spawn_surface_box, spawn_world};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle sport_bike", true);
    app.insert_resource(GroundVehicleExamplePane {
        camera_distance: 8.0,
        camera_height: 3.5,
        peak_torque_nm: 115.0,
        ..default()
    });
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);
    spawn_overlay(&mut commands, "ground_vehicle sport_bike");

    // Winding slalom cones
    let cone_surface = GroundVehicleSurface::default();
    for i in 0..8 {
        let z = -10.0 - i as f32 * 12.0;
        let x = if i % 2 == 0 { 4.0 } else { -4.0 };
        spawn_surface_box(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Slalom Cone {}", i + 1),
            Vec3::new(0.6, 1.2, 0.6),
            Transform::from_xyz(x, 0.6, z),
            Color::srgb(0.95, 0.55, 0.05),
            cone_surface,
        );
    }

    // Speed ramp
    spawn_surface_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Speed Ramp",
        Vec3::new(4.0, 0.3, 8.0),
        Transform::from_xyz(0.0, 0.15, -120.0)
            .with_rotation(Quat::from_rotation_x(-8.0_f32.to_radians())),
        Color::srgb(0.45, 0.45, 0.50),
        cone_surface,
    );

    support::spawn_sport_bike_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Sport Bike",
        Transform::from_xyz(0.0, 1.0, 8.0),
        true,
    );
}
