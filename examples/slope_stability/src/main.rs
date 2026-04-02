use ground_vehicle_example_support as support;

use bevy::prelude::*;
use support::{spawn_overlay, spawn_ramp, spawn_rover_demo, spawn_world};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle slope_stability", true);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Main Ramp",
        Vec3::new(10.0, 0.8, 28.0),
        Transform::from_xyz(0.0, 1.4, 0.0).with_rotation(Quat::from_rotation_x(-0.26)),
        Color::srgb(0.46, 0.38, 0.23),
        default(),
    );
    spawn_ramp(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Off Camber Pad",
        Vec3::new(8.0, 0.6, 14.0),
        Transform::from_xyz(-10.0, 0.9, -16.0)
            .with_rotation(Quat::from_rotation_x(-0.18) * Quat::from_rotation_z(0.14)),
        Color::srgb(0.36, 0.34, 0.19),
        default(),
    );
    spawn_rover_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Slope Rover",
        Transform::from_xyz(0.0, 3.2, 8.0),
        true,
    );
    spawn_overlay(&mut commands, "ground_vehicle slope_stability");
}
