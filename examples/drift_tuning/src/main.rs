use ground_vehicle_example_support as support;

use bevy::prelude::*;
use ground_vehicle::GroundVehicleSurface;
use support::{spawn_drift_coupe_demo, spawn_overlay, spawn_surface_box, spawn_world};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle drift_tuning", true);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);
    spawn_surface_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Drift Pad",
        Vec3::new(42.0, 0.06, 42.0),
        Transform::from_xyz(0.0, 0.03, 0.0),
        Color::srgb(0.13, 0.14, 0.16),
        GroundVehicleSurface {
            longitudinal_grip_scale: 0.98,
            lateral_grip_scale: 0.90,
            ..default()
        },
    );
    spawn_drift_coupe_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Street Drift Coupe",
        Transform::from_xyz(0.0, 1.18, 16.0),
        true,
    );
    spawn_overlay(&mut commands, "ground_vehicle drift_tuning");
}
