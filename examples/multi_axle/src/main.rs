use ground_vehicle_example_support as support;

use bevy::prelude::*;
use support::{spawn_bump_strip, spawn_cargo_truck_demo, spawn_overlay, spawn_world};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle multi_axle", false);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);
    spawn_bump_strip(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Truck Bump",
        Vec3::new(0.0, 0.14, 10.0),
        6,
        3.0,
    );
    spawn_cargo_truck_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Cargo Truck",
        Transform::from_xyz(0.0, 1.65, 20.0),
        true,
    );
    spawn_overlay(&mut commands, "ground_vehicle multi_axle");
}
