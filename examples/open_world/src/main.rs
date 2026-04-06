//! Open world example — GTA-like heavy sedan with strong stability aids.
//!
//! Demonstrates a heavy, easy-to-drive sedan suited for open world games.
//! Strong yaw stability prevents spinning, high airborne self-righting lets
//! the car survive jumps, and the forgiving grip makes casual driving fun.
//! The environment includes ramps, obstacles, and varied surfaces.

use avian3d::prelude::*;
use bevy::prelude::*;
use ground_vehicle::GroundVehicleSurface;
use ground_vehicle_example_support as support;
use support::{spawn_overlay, spawn_surface_box, spawn_world};

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle open_world", true);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);
    spawn_overlay(&mut commands, "ground_vehicle open_world");

    // Street-like road markings
    for i in 0..20 {
        let z = -4.0 - i as f32 * 8.0;
        spawn_surface_box(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Road Dash {}", i + 1),
            Vec3::new(0.15, 0.02, 3.0),
            Transform::from_xyz(0.0, 0.01, z),
            Color::srgb(0.95, 0.92, 0.75),
            GroundVehicleSurface::default(),
        );
    }

    // Stunt ramp — big jump
    spawn_surface_box(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Stunt Ramp",
        Vec3::new(6.0, 0.8, 8.0),
        Transform::from_xyz(0.0, 0.4, -60.0)
            .with_rotation(Quat::from_rotation_x(-15.0_f32.to_radians())),
        Color::srgb(0.60, 0.58, 0.55),
        GroundVehicleSurface::default(),
    );

    // Destructible-style crates (static but small, satisfying to hit)
    let crate_positions = [
        Vec3::new(5.0, 0.5, -25.0),
        Vec3::new(5.5, 0.5, -26.5),
        Vec3::new(4.5, 0.5, -26.0),
        Vec3::new(5.0, 1.5, -25.8),
        Vec3::new(-6.0, 0.5, -40.0),
        Vec3::new(-5.5, 0.5, -41.0),
        Vec3::new(-6.5, 0.5, -40.5),
    ];
    for (i, pos) in crate_positions.iter().enumerate() {
        commands.spawn((
            Name::new(format!("Crate {}", i + 1)),
            RigidBody::Dynamic,
            Mass(80.0),
            Collider::cuboid(1.0, 1.0, 1.0),
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.65, 0.45, 0.22),
                perceptual_roughness: 0.95,
                ..default()
            })),
            Transform::from_translation(*pos),
        ));
    }

    // Parking bollards (thin static pillars)
    for i in 0..6 {
        let x = -3.0 + i as f32 * 1.2;
        commands.spawn((
            Name::new(format!("Bollard {}", i + 1)),
            RigidBody::Dynamic,
            Mass(15.0),
            Collider::cylinder(0.15, 0.9),
            Mesh3d(meshes.add(Cylinder::new(0.15, 0.9))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.85, 0.72, 0.10),
                perceptual_roughness: 0.70,
                metallic: 0.40,
                ..default()
            })),
            Transform::from_xyz(x, 0.45, -15.0),
        ));
    }

    // Sidewalk curbs
    for side in [-1.0_f32, 1.0] {
        spawn_surface_box(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Sidewalk {}", if side < 0.0 { "L" } else { "R" }),
            Vec3::new(3.0, 0.15, 160.0),
            Transform::from_xyz(side * 7.5, 0.075, -60.0),
            Color::srgb(0.65, 0.63, 0.60),
            GroundVehicleSurface::default(),
        );
    }

    // Grass areas with reduced grip
    let grass_surface = GroundVehicleSurface {
        longitudinal_grip_scale: 0.60,
        lateral_grip_scale: 0.50,
        rolling_drag_scale: 3.0,
        brake_scale: 0.55,
    };
    for side in [-1.0_f32, 1.0] {
        spawn_surface_box(
            &mut commands,
            &mut meshes,
            &mut materials,
            &format!("Grass {}", if side < 0.0 { "L" } else { "R" }),
            Vec3::new(8.0, 0.03, 160.0),
            Transform::from_xyz(side * 14.0, 0.015, -60.0),
            Color::srgb(0.30, 0.52, 0.18),
            grass_surface,
        );
    }

    support::spawn_open_world_sedan_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Open World Sedan",
        Transform::from_xyz(0.0, 1.2, 12.0),
        true,
    );
}
