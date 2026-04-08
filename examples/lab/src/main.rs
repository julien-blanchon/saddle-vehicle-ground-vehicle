#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;
use ground_vehicle_example_support as support;

use avian3d::prelude::{Collider, Mass, RigidBody, TransformInterpolation};
use bevy::prelude::*;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use bevy_brp_extras::BrpExtrasPlugin;
use bevy_enhanced_input::{
    context::InputContextAppExt,
    prelude::{Action, InputAction, Press as InputPress, Start, actions, bindings},
};
use ground_vehicle::GroundVehicleSurface;
use support::{
    ExampleDriver, FollowCamera, set_camera_preset, spawn_bump_strip, spawn_cargo_truck_demo,
    spawn_compact_car_demo, spawn_drift_coupe_demo, spawn_kart_demo, spawn_open_world_sedan_demo,
    spawn_overlay, spawn_ramp, spawn_rover_demo, spawn_sim_racer_demo, spawn_skid_vehicle_demo,
    spawn_sport_bike_demo, spawn_surface_box, spawn_world,
};

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
const DEFAULT_LAB_BRP_PORT: u16 = 15_712;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LabState {
    pub compact: Entity,
    pub drift: Entity,
    pub truck: Entity,
    pub skid: Entity,
    pub rover: Entity,
    pub sport_bike: Entity,
    pub sim_racer: Entity,
    pub kart: Entity,
    pub sedan: Entity,
    pub active: ActiveVehicle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveVehicle {
    Compact,
    Drift,
    Truck,
    Skid,
    Rover,
    SportBike,
    SimRacer,
    Kart,
    Sedan,
}

#[derive(Component)]
struct LabSwitcher;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct SelectCompactAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct SelectDriftAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct SelectTruckAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct SelectSkidAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct SelectRoverAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct SelectSportBikeAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct SelectSimRacerAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct SelectKartAction;

#[derive(Debug, InputAction)]
#[action_output(bool)]
struct SelectSedanAction;

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle lab", true);
    app.add_input_context::<LabSwitcher>()
        .add_observer(select_compact)
        .add_observer(select_drift)
        .add_observer(select_truck)
        .add_observer(select_skid)
        .add_observer(select_rover)
        .add_observer(select_sport_bike)
        .add_observer(select_sim_racer)
        .add_observer(select_kart)
        .add_observer(select_sedan);
    #[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
    app.add_plugins((
        RemotePlugin::default(),
        BrpExtrasPlugin::with_http_plugin(RemoteHttpPlugin::default().with_port(lab_brp_port())),
    ));
    #[cfg(feature = "e2e")]
    app.add_plugins(e2e::GroundVehicleLabE2EPlugin);
    app.add_systems(Startup, setup);
    app.run();
}

#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
fn lab_brp_port() -> u16 {
    std::env::var("GROUND_VEHICLE_LAB_BRP_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_LAB_BRP_PORT)
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
        "Lab Slope Ramp",
        Vec3::new(12.0, 1.0, 34.0),
        Transform::from_xyz(42.0, 1.9, 44.0).with_rotation(Quat::from_rotation_x(-0.28)),
        Color::srgb(0.45, 0.37, 0.21),
        default(),
    );
    spawn_bump_strip(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Truck Bump",
        Vec3::new(-46.0, 0.14, 10.0),
        8,
        3.0,
    );
    spawn_lab_sport_bike_zone(&mut commands, &mut meshes, &mut materials);
    spawn_lab_sim_racing_zone(&mut commands, &mut meshes, &mut materials);
    spawn_lab_kart_zone(&mut commands, &mut meshes, &mut materials);
    spawn_lab_open_world_zone(&mut commands, &mut meshes, &mut materials);

    let compact = spawn_compact_car_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Compact Car",
        Transform::from_xyz(0.0, 1.25, 18.0),
        true,
    );
    let drift = spawn_drift_coupe_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Drift Coupe",
        Transform::from_xyz(42.0, 1.18, 10.0),
        true,
    );
    let truck = spawn_cargo_truck_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Cargo Truck",
        Transform::from_xyz(-46.0, 1.7, 20.0),
        true,
    );
    let skid = spawn_skid_vehicle_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Skid Vehicle",
        Transform::from_xyz(0.0, 1.35, -42.0),
        true,
    );
    let rover = spawn_rover_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Slope Rover",
        Transform::from_xyz(42.0, 4.2, 58.0),
        true,
    );
    let sport_bike = spawn_sport_bike_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Sport Bike",
        Transform::from_xyz(-82.0, 1.0, 58.0),
        true,
    );
    let sim_racer = spawn_sim_racer_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Sim Racer",
        Transform::from_xyz(82.0, 1.0, 22.0),
        true,
    );
    let kart = spawn_kart_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Kart",
        Transform::from_xyz(-82.0, 0.8, -18.0),
        true,
    );
    let sedan = spawn_open_world_sedan_demo(
        &mut commands,
        &mut meshes,
        &mut materials,
        "Lab Open World Sedan",
        Transform::from_xyz(82.0, 1.2, -42.0),
        true,
    );

    commands
        .entity(compact)
        .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::ACTIVE);
    commands
        .entity(drift)
        .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::INACTIVE);
    commands
        .entity(truck)
        .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::INACTIVE);
    commands
        .entity(skid)
        .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::INACTIVE);
    commands
        .entity(rover)
        .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::INACTIVE);
    commands
        .entity(sport_bike)
        .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::INACTIVE);
    commands
        .entity(sim_racer)
        .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::INACTIVE);
    commands
        .entity(kart)
        .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::INACTIVE);
    commands
        .entity(sedan)
        .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::INACTIVE);

    commands.insert_resource(LabState {
        compact,
        drift,
        truck,
        skid,
        rover,
        sport_bike,
        sim_racer,
        kart,
        sedan,
        active: ActiveVehicle::Compact,
    });
    commands.spawn((
        Name::new("Lab Input"),
        LabSwitcher,
        actions!(LabSwitcher[
            (
                Action::<SelectCompactAction>::new(),
                InputPress::default(),
                bindings![KeyCode::Digit1],
            ),
            (
                Action::<SelectDriftAction>::new(),
                InputPress::default(),
                bindings![KeyCode::Digit2],
            ),
            (
                Action::<SelectTruckAction>::new(),
                InputPress::default(),
                bindings![KeyCode::Digit3],
            ),
            (
                Action::<SelectSkidAction>::new(),
                InputPress::default(),
                bindings![KeyCode::Digit4],
            ),
            (
                Action::<SelectRoverAction>::new(),
                InputPress::default(),
                bindings![KeyCode::Digit5],
            ),
            (
                Action::<SelectSportBikeAction>::new(),
                InputPress::default(),
                bindings![KeyCode::Digit6],
            ),
            (
                Action::<SelectSimRacerAction>::new(),
                InputPress::default(),
                bindings![KeyCode::Digit7],
            ),
            (
                Action::<SelectKartAction>::new(),
                InputPress::default(),
                bindings![KeyCode::Digit8],
            ),
            (
                Action::<SelectSedanAction>::new(),
                InputPress::default(),
                bindings![KeyCode::Digit9],
            )
        ]),
    ));

    spawn_overlay(&mut commands, "ground_vehicle lab");
}

fn spawn_lab_sport_bike_zone(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for index in 0..8 {
        let z = 40.0 - index as f32 * 12.0;
        let x = -82.0 + if index % 2 == 0 { 4.0 } else { -4.0 };
        spawn_surface_box(
            commands,
            meshes,
            materials,
            &format!("Lab Slalom Cone {}", index + 1),
            Vec3::new(0.6, 1.2, 0.6),
            Transform::from_xyz(x, 0.6, z),
            Color::srgb(0.95, 0.55, 0.05),
            GroundVehicleSurface::default(),
        );
    }

    spawn_surface_box(
        commands,
        meshes,
        materials,
        "Lab Sport Bike Ramp",
        Vec3::new(4.0, 0.3, 8.0),
        Transform::from_xyz(-82.0, 0.15, -40.0)
            .with_rotation(Quat::from_rotation_x(-8.0_f32.to_radians())),
        Color::srgb(0.45, 0.45, 0.50),
        GroundVehicleSurface::default(),
    );
}

fn spawn_lab_sim_racing_zone(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let wall_surface = GroundVehicleSurface {
        lateral_grip_scale: 0.3,
        ..default()
    };
    for side in [-1.0_f32, 1.0] {
        for segment in 0..5 {
            let z = 20.0 - 36.0 * segment as f32;
            spawn_surface_box(
                commands,
                meshes,
                materials,
                &format!(
                    "Lab Sim Wall {} {}",
                    if side < 0.0 { "L" } else { "R" },
                    segment + 1
                ),
                Vec3::new(0.4, 0.8, 36.0),
                Transform::from_xyz(82.0 + side * 8.0, 0.4, z),
                Color::srgb(0.55, 0.55, 0.58),
                wall_surface,
            );
        }
    }

    spawn_surface_box(
        commands,
        meshes,
        materials,
        "Lab Sim Braking Zone",
        Vec3::new(12.0, 0.02, 2.0),
        Transform::from_xyz(82.0, 0.01, -120.0),
        Color::srgb(0.85, 0.70, 0.10),
        GroundVehicleSurface::default(),
    );
}

fn spawn_lab_kart_zone(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_surface_box(
        commands,
        meshes,
        materials,
        "Lab Kart Boost Pad",
        Vec3::new(3.0, 0.04, 6.0),
        Transform::from_xyz(-82.0, 0.02, -34.0),
        Color::srgb(0.15, 0.80, 0.95),
        GroundVehicleSurface {
            longitudinal_grip_scale: 1.8,
            lateral_grip_scale: 1.4,
            ..default()
        },
    );

    spawn_surface_box(
        commands,
        meshes,
        materials,
        "Lab Kart Jump Ramp",
        Vec3::new(4.0, 0.5, 4.0),
        Transform::from_xyz(-82.0, 0.25, -62.0)
            .with_rotation(Quat::from_rotation_x(-12.0_f32.to_radians())),
        Color::srgb(0.90, 0.75, 0.10),
        GroundVehicleSurface::default(),
    );

    spawn_surface_box(
        commands,
        meshes,
        materials,
        "Lab Kart Oil Patch",
        Vec3::new(6.0, 0.02, 5.0),
        Transform::from_xyz(-78.0, 0.01, -78.0),
        Color::srgb(0.15, 0.12, 0.10),
        GroundVehicleSurface {
            longitudinal_grip_scale: 0.35,
            lateral_grip_scale: 0.25,
            brake_scale: 0.30,
            ..default()
        },
    );
}

fn spawn_lab_open_world_zone(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    spawn_surface_box(
        commands,
        meshes,
        materials,
        "Lab Sedan Stunt Ramp",
        Vec3::new(6.0, 0.8, 8.0),
        Transform::from_xyz(82.0, 0.4, -88.0)
            .with_rotation(Quat::from_rotation_x(-15.0_f32.to_radians())),
        Color::srgb(0.60, 0.58, 0.55),
        GroundVehicleSurface::default(),
    );

    for (index, position) in [
        Vec3::new(87.0, 0.5, -54.0),
        Vec3::new(87.5, 0.5, -55.5),
        Vec3::new(86.5, 0.5, -55.0),
    ]
    .into_iter()
    .enumerate()
    {
        commands.spawn((
            Name::new(format!("Lab Sedan Crate {}", index + 1)),
            RigidBody::Dynamic,
            Mass(80.0),
            Collider::cuboid(1.0, 1.0, 1.0),
            TransformInterpolation,
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.65, 0.45, 0.22),
                perceptual_roughness: 0.95,
                ..default()
            })),
            Transform::from_translation(position),
        ));
    }

    for index in 0..4 {
        let x = 79.0 + index as f32 * 1.2;
        commands.spawn((
            Name::new(format!("Lab Sedan Bollard {}", index + 1)),
            RigidBody::Dynamic,
            Mass(15.0),
            Collider::cylinder(0.15, 0.9),
            TransformInterpolation,
            Mesh3d(meshes.add(Cylinder::new(0.15, 0.9))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.85, 0.72, 0.10),
                perceptual_roughness: 0.70,
                metallic: 0.40,
                ..default()
            })),
            Transform::from_xyz(x, 0.45, -66.0),
        ));
    }
}

fn select_compact(
    _: On<Start<SelectCompactAction>>,
    mut commands: Commands,
    mut state: ResMut<LabState>,
    mut cameras: Query<&mut FollowCamera>,
) {
    set_active_vehicle(
        &mut commands,
        &mut state,
        &mut cameras,
        ActiveVehicle::Compact,
    );
}

fn select_drift(
    _: On<Start<SelectDriftAction>>,
    mut commands: Commands,
    mut state: ResMut<LabState>,
    mut cameras: Query<&mut FollowCamera>,
) {
    set_active_vehicle(
        &mut commands,
        &mut state,
        &mut cameras,
        ActiveVehicle::Drift,
    );
}

fn select_truck(
    _: On<Start<SelectTruckAction>>,
    mut commands: Commands,
    mut state: ResMut<LabState>,
    mut cameras: Query<&mut FollowCamera>,
) {
    set_active_vehicle(
        &mut commands,
        &mut state,
        &mut cameras,
        ActiveVehicle::Truck,
    );
}

fn select_skid(
    _: On<Start<SelectSkidAction>>,
    mut commands: Commands,
    mut state: ResMut<LabState>,
    mut cameras: Query<&mut FollowCamera>,
) {
    set_active_vehicle(&mut commands, &mut state, &mut cameras, ActiveVehicle::Skid);
}

fn select_rover(
    _: On<Start<SelectRoverAction>>,
    mut commands: Commands,
    mut state: ResMut<LabState>,
    mut cameras: Query<&mut FollowCamera>,
) {
    set_active_vehicle(
        &mut commands,
        &mut state,
        &mut cameras,
        ActiveVehicle::Rover,
    );
}

fn select_sport_bike(
    _: On<Start<SelectSportBikeAction>>,
    mut commands: Commands,
    mut state: ResMut<LabState>,
    mut cameras: Query<&mut FollowCamera>,
) {
    set_active_vehicle(
        &mut commands,
        &mut state,
        &mut cameras,
        ActiveVehicle::SportBike,
    );
}

fn select_sim_racer(
    _: On<Start<SelectSimRacerAction>>,
    mut commands: Commands,
    mut state: ResMut<LabState>,
    mut cameras: Query<&mut FollowCamera>,
) {
    set_active_vehicle(
        &mut commands,
        &mut state,
        &mut cameras,
        ActiveVehicle::SimRacer,
    );
}

fn select_kart(
    _: On<Start<SelectKartAction>>,
    mut commands: Commands,
    mut state: ResMut<LabState>,
    mut cameras: Query<&mut FollowCamera>,
) {
    set_active_vehicle(&mut commands, &mut state, &mut cameras, ActiveVehicle::Kart);
}

fn select_sedan(
    _: On<Start<SelectSedanAction>>,
    mut commands: Commands,
    mut state: ResMut<LabState>,
    mut cameras: Query<&mut FollowCamera>,
) {
    set_active_vehicle(
        &mut commands,
        &mut state,
        &mut cameras,
        ActiveVehicle::Sedan,
    );
}

fn set_active_vehicle(
    commands: &mut Commands,
    state: &mut LabState,
    cameras: &mut Query<&mut FollowCamera>,
    active: ActiveVehicle,
) {
    if state.active == active {
        return;
    }

    state.active = active;
    for entity in [
        state.compact,
        state.drift,
        state.truck,
        state.skid,
        state.rover,
        state.sport_bike,
        state.sim_racer,
        state.kart,
        state.sedan,
    ] {
        commands
            .entity(entity)
            .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::INACTIVE);
    }

    let (target, distance, height, lateral) = match active {
        ActiveVehicle::Compact => (state.compact, 11.5, 4.8, 0.0),
        ActiveVehicle::Drift => (state.drift, 10.5, 4.3, -1.4),
        ActiveVehicle::Truck => (state.truck, 15.0, 6.8, 0.0),
        ActiveVehicle::Skid => (state.skid, 13.0, 5.6, -0.8),
        ActiveVehicle::Rover => (state.rover, 9.5, 5.0, 0.6),
        ActiveVehicle::SportBike => (state.sport_bike, 8.0, 3.5, 0.0),
        ActiveVehicle::SimRacer => (state.sim_racer, 9.0, 3.8, 0.0),
        ActiveVehicle::Kart => (state.kart, 7.0, 3.2, 0.0),
        ActiveVehicle::Sedan => (state.sedan, 11.5, 5.0, 0.0),
    };
    commands
        .entity(target)
        .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::ACTIVE);
    set_camera_preset(cameras, distance, height, lateral);
}
