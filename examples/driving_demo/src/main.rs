#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;

use ground_vehicle_example_support as support;

use bevy::prelude::*;
use ground_vehicle::{GroundVehicleSurface, GroundVehicleTelemetry};
use support::{spawn_drift_coupe_demo, spawn_overlay, spawn_ramp, spawn_surface_box, spawn_world};

#[derive(Component)]
struct DrivingDemoPlayer;

#[derive(Component)]
struct DrivingHud;

#[derive(Component)]
struct CheckpointGate {
    index: usize,
    radius: f32,
}

#[derive(Resource, Debug)]
struct DrivingDemoProgress {
    next_checkpoint: usize,
    checkpoint_count: usize,
    lap_started_at: f32,
    laps_completed: u32,
    best_lap_seconds: Option<f32>,
}

impl Default for DrivingDemoProgress {
    fn default() -> Self {
        Self {
            next_checkpoint: 0,
            checkpoint_count: 4,
            lap_started_at: 0.0,
            laps_completed: 0,
            best_lap_seconds: None,
        }
    }
}

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle driving_demo", true);
    app.init_resource::<DrivingDemoProgress>();
    #[cfg(feature = "e2e")]
    {
        app.add_plugins(e2e::DrivingDemoE2EPlugin);
    }
    app.add_systems(Startup, setup)
        .add_systems(Update, (track_checkpoint_progress, update_hud).chain());
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut progress: ResMut<DrivingDemoProgress>,
    time: Res<Time>,
) {
    spawn_world(&mut commands, &mut meshes, &mut materials);
    spawn_track(&mut commands, &mut meshes, &mut materials);

    let player = spawn_drift_coupe_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Checkpoint Runner",
        Transform::from_xyz(-34.0, 1.18, 38.0).with_rotation(Quat::from_rotation_y(0.28)),
        true,
    );
    commands.entity(player).insert(DrivingDemoPlayer);

    spawn_overlay(&mut commands, "ground_vehicle driving_demo");
    spawn_driving_hud(&mut commands);

    progress.lap_started_at = time.elapsed_secs();
}

fn spawn_track(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_surface_box(
        commands,
        meshes,
        materials,
        "Stage Asphalt Long",
        Vec3::new(18.0, 0.05, 96.0),
        Transform::from_xyz(-20.0, 0.03, -6.0),
        Color::srgb(0.12, 0.13, 0.15),
        GroundVehicleSurface {
            longitudinal_grip_scale: 1.02,
            lateral_grip_scale: 0.96,
            ..default()
        },
    );
    spawn_surface_box(
        commands,
        meshes,
        materials,
        "Stage Asphalt East",
        Vec3::new(74.0, 0.05, 18.0),
        Transform::from_xyz(6.0, 0.03, -62.0),
        Color::srgb(0.12, 0.13, 0.15),
        GroundVehicleSurface {
            longitudinal_grip_scale: 1.02,
            lateral_grip_scale: 0.96,
            ..default()
        },
    );
    spawn_surface_box(
        commands,
        meshes,
        materials,
        "Stage Asphalt Return",
        Vec3::new(18.0, 0.05, 104.0),
        Transform::from_xyz(34.0, 0.03, -8.0),
        Color::srgb(0.12, 0.13, 0.15),
        GroundVehicleSurface {
            longitudinal_grip_scale: 1.02,
            lateral_grip_scale: 0.96,
            ..default()
        },
    );
    spawn_surface_box(
        commands,
        meshes,
        materials,
        "Stage Asphalt Finish",
        Vec3::new(66.0, 0.05, 18.0),
        Transform::from_xyz(0.0, 0.03, 42.0),
        Color::srgb(0.12, 0.13, 0.15),
        GroundVehicleSurface {
            longitudinal_grip_scale: 1.02,
            lateral_grip_scale: 0.96,
            ..default()
        },
    );

    for (name, translation, size, color) in [
        (
            "West Canyon Wall",
            Vec3::new(-51.0, 6.0, -8.0),
            Vec3::new(6.0, 12.0, 118.0),
            Color::srgb(0.44, 0.31, 0.19),
        ),
        (
            "East Canyon Wall",
            Vec3::new(51.0, 7.0, -6.0),
            Vec3::new(6.0, 14.0, 128.0),
            Color::srgb(0.46, 0.34, 0.21),
        ),
        (
            "North Ridge",
            Vec3::new(-4.0, 5.0, -82.0),
            Vec3::new(104.0, 10.0, 8.0),
            Color::srgb(0.39, 0.30, 0.18),
        ),
        (
            "South Ridge",
            Vec3::new(-4.0, 4.5, 58.0),
            Vec3::new(104.0, 9.0, 8.0),
            Color::srgb(0.36, 0.28, 0.18),
        ),
    ] {
        spawn_surface_box(
            commands,
            meshes,
            materials,
            name,
            size,
            Transform::from_translation(translation),
            color,
            GroundVehicleSurface::default(),
        );
    }

    spawn_ramp(
        commands,
        meshes,
        materials,
        "Jump Ramp",
        Vec3::new(10.0, 0.8, 18.0),
        Transform::from_xyz(32.0, 1.3, 18.0).with_rotation(Quat::from_rotation_x(-0.18)),
        Color::srgb(0.28, 0.24, 0.16),
        GroundVehicleSurface {
            longitudinal_grip_scale: 1.08,
            lateral_grip_scale: 0.88,
            ..default()
        },
    );

    for (name, translation, size, color) in [
        (
            "Spectator Tower",
            Vec3::new(-28.0, 6.0, 60.0),
            Vec3::new(5.0, 12.0, 5.0),
            Color::srgb(0.24, 0.26, 0.29),
        ),
        (
            "Pit Garage",
            Vec3::new(22.0, 3.5, 60.0),
            Vec3::new(18.0, 7.0, 12.0),
            Color::srgb(0.21, 0.23, 0.26),
        ),
        (
            "Cargo Stack",
            Vec3::new(-36.0, 1.6, -58.0),
            Vec3::new(10.0, 3.2, 4.2),
            Color::srgb(0.71, 0.33, 0.21),
        ),
        (
            "Service Trailer",
            Vec3::new(38.0, 1.4, -50.0),
            Vec3::new(8.0, 2.8, 3.8),
            Color::srgb(0.72, 0.74, 0.78),
        ),
    ] {
        spawn_surface_box(
            commands,
            meshes,
            materials,
            name,
            size,
            Transform::from_translation(translation),
            color,
            GroundVehicleSurface::default(),
        );
    }

    let checkpoints = [
        ("Checkpoint 1", Vec3::new(-32.0, 0.0, -40.0)),
        ("Checkpoint 2", Vec3::new(20.0, 0.0, -62.0)),
        ("Checkpoint 3", Vec3::new(34.0, 0.0, 6.0)),
        ("Checkpoint 4", Vec3::new(-10.0, 0.0, 42.0)),
    ];

    for (index, (name, center)) in checkpoints.into_iter().enumerate() {
        spawn_checkpoint_gate(commands, meshes, materials, name, index, center);
    }
}

fn spawn_checkpoint_gate(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    name: &str,
    index: usize,
    center: Vec3,
) {
    let accent = checkpoint_color(index);
    for (side, x_offset) in [("Left", -4.0), ("Right", 4.0)] {
        commands.spawn((
            Name::new(format!("{name} {side} Pylon")),
            Mesh3d(meshes.add(Cuboid::new(0.8, 5.6, 0.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: accent,
                emissive: accent.to_linear() * 0.10,
                perceptual_roughness: 0.48,
                ..default()
            })),
            Transform::from_xyz(center.x + x_offset, 2.8, center.z),
        ));
    }

    commands.spawn((
        Name::new(format!("{name} Arch")),
        CheckpointGate { index, radius: 7.5 },
        Mesh3d(meshes.add(Cuboid::new(9.2, 0.6, 0.9))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.92, 0.94, 0.98),
            emissive: accent.to_linear() * 0.06,
            perceptual_roughness: 0.18,
            ..default()
        })),
        Transform::from_xyz(center.x, 5.5, center.z),
    ));

    commands.spawn((
        Name::new(format!("{name} Beacon")),
        PointLight {
            intensity: 32_000.0,
            range: 18.0,
            color: accent,
            ..default()
        },
        Transform::from_xyz(center.x, 6.8, center.z),
    ));
}

fn spawn_driving_hud(commands: &mut Commands) {
    commands.spawn((
        Name::new("Driving Demo HUD"),
        DrivingHud,
        Text::new(""),
        Node {
            position_type: PositionType::Absolute,
            right: px(18.0),
            top: px(16.0),
            ..default()
        },
        TextFont {
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

fn track_checkpoint_progress(
    time: Res<Time>,
    mut progress: ResMut<DrivingDemoProgress>,
    player: Query<&Transform, With<DrivingDemoPlayer>>,
    checkpoints: Query<(&CheckpointGate, &Transform)>,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };

    let Some((gate, gate_transform)) = checkpoints
        .iter()
        .find(|(gate, _)| gate.index == progress.next_checkpoint)
    else {
        return;
    };

    let distance = player_transform
        .translation
        .distance(gate_transform.translation);
    if distance > gate.radius {
        return;
    }

    progress.next_checkpoint += 1;
    if progress.next_checkpoint < progress.checkpoint_count {
        return;
    }

    progress.next_checkpoint = 0;
    progress.laps_completed += 1;
    let lap_seconds = time.elapsed_secs() - progress.lap_started_at;
    progress.best_lap_seconds = Some(
        progress
            .best_lap_seconds
            .map_or(lap_seconds, |best| best.min(lap_seconds)),
    );
    progress.lap_started_at = time.elapsed_secs();
}

fn update_hud(
    time: Res<Time>,
    progress: Res<DrivingDemoProgress>,
    player: Query<&GroundVehicleTelemetry, With<DrivingDemoPlayer>>,
    mut hud: Query<&mut Text, With<DrivingHud>>,
) {
    let Ok(mut hud) = hud.single_mut() else {
        return;
    };
    let Ok(player) = player.single() else {
        return;
    };

    let lap_time = time.elapsed_secs() - progress.lap_started_at;
    let best = progress
        .best_lap_seconds
        .map_or("--".to_string(), |seconds| format!("{seconds:>5.1}s"));

    hud.0 = format!(
        "Checkpoint Run\nLap {}  Next gate: {}/{}\nCurrent {:>4.1}s  Best {}\nPlayer speed {:>5.1} m/s  Gear {}\nObjective: clear all gates and hit the jump.",
        progress.laps_completed + 1,
        progress.next_checkpoint + 1,
        progress.checkpoint_count,
        lap_time,
        best,
        player.speed_mps,
        player.selected_gear,
    );
}

fn checkpoint_color(index: usize) -> Color {
    match index {
        0 => Color::srgb(0.96, 0.53, 0.22),
        1 => Color::srgb(0.28, 0.78, 0.95),
        2 => Color::srgb(0.53, 0.90, 0.43),
        _ => Color::srgb(0.98, 0.86, 0.34),
    }
}
