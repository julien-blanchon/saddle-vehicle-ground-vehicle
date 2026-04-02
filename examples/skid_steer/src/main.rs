use ground_vehicle_example_support as support;

use bevy::prelude::*;
use ground_vehicle::GroundVehicleSurface;
use support::{spawn_overlay, spawn_skid_vehicle_demo, spawn_surface_box, spawn_world};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle skid_steer", true);
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
        "Skid Pad",
        Vec3::new(36.0, 0.05, 36.0),
        Transform::from_xyz(0.0, 0.025, 0.0),
        Color::srgb(0.20, 0.23, 0.20),
        GroundVehicleSurface {
            rolling_drag_scale: 1.15,
            ..default()
        },
    );
    spawn_skid_vehicle_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Skid Vehicle",
        Transform::from_xyz(0.0, 1.30, 12.0),
        true,
    );
    spawn_overlay(&mut commands, "ground_vehicle skid_steer");
}
