use ground_vehicle_example_support as support;

use bevy::prelude::*;
use support::{spawn_compact_car_demo, spawn_overlay, spawn_world};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle basic", false);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);
    spawn_compact_car_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Basic Hatchback",
        Transform::from_xyz(0.0, 1.25, 18.0),
        true,
    );
    spawn_overlay(&mut commands, "ground_vehicle basic");
}
