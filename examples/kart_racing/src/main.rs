//! Kart racing example — playful Mario Kart-inspired arcade kart.
//!
//! Demonstrates a lightweight, snappy kart with exaggerated grip, instant
//! direction changes, and strong self-righting. The handling is forgiving
//! and fun: high top speed, easy drifts via handbrake, and ramp jumps.

use bevy::prelude::*;
use ground_vehicle::GroundVehicleSurface;
use ground_vehicle_example_support as support;
use support::{GroundVehicleExamplePane, spawn_overlay, spawn_surface_box, spawn_world};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle kart_racing", true);
    app.insert_resource(GroundVehicleExamplePane {
        camera_distance: 7.0,
        camera_height: 3.2,
        peak_torque_nm: 210.0,
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
    spawn_overlay(&mut commands, "ground_vehicle kart_racing");

    // Boost pad — high grip for speed
    let boost_surface = GroundVehicleSurface {
        longitudinal_grip_scale: 1.8,
        lateral_grip_scale: 1.4,
        ..default()
    };
    spawn_surface_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Boost Pad",
        Vec3::new(3.0, 0.04, 6.0),
        Transform::from_xyz(0.0, 0.02, -12.0),
        Color::srgb(0.15, 0.80, 0.95),
        boost_surface,
    );

    // Jump ramps
    for (i, z) in [-35.0_f32, -70.0].iter().enumerate() {
        spawn_surface_box(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Jump Ramp {}", i + 1),
            Vec3::new(4.0, 0.5, 4.0),
            Transform::from_xyz(0.0, 0.25, *z)
                .with_rotation(Quat::from_rotation_x(-12.0_f32.to_radians())),
            Color::srgb(0.90, 0.75, 0.10),
            GroundVehicleSurface::default(),
        );
    }

    // Slippery oil patch
    let oil_surface = GroundVehicleSurface {
        longitudinal_grip_scale: 0.35,
        lateral_grip_scale: 0.25,
        brake_scale: 0.30,
        ..default()
    };
    spawn_surface_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Oil Patch",
        Vec3::new(6.0, 0.02, 5.0),
        Transform::from_xyz(3.0, 0.01, -50.0),
        Color::srgb(0.15, 0.12, 0.10),
        oil_surface,
    );

    // Dirt shortcut
    let dirt_surface = GroundVehicleSurface {
        longitudinal_grip_scale: 0.70,
        lateral_grip_scale: 0.55,
        rolling_drag_scale: 2.5,
        ..default()
    };
    spawn_surface_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Dirt Shortcut",
        Vec3::new(5.0, 0.03, 15.0),
        Transform::from_xyz(-8.0, 0.015, -25.0),
        Color::srgb(0.52, 0.38, 0.22),
        dirt_surface,
    );

    // Barriers around the play area
    for side in [-1.0_f32, 1.0] {
        spawn_surface_box(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Barrier {}", if side < 0.0 { "L" } else { "R" }),
            Vec3::new(0.3, 0.6, 120.0),
            Transform::from_xyz(side * 14.0, 0.3, -40.0),
            Color::srgb(0.75, 0.20, 0.15),
            GroundVehicleSurface::default(),
        );
    }

    support::spawn_kart_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Racing Kart",
        Transform::from_xyz(0.0, 0.8, 8.0),
        true,
    );
}
