#[cfg(feature = "e2e")]
mod e2e;
#[cfg(feature = "e2e")]
mod scenarios;
use ground_vehicle_example_support as support;

use bevy::prelude::*;
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use bevy::remote::{RemotePlugin, http::RemoteHttpPlugin};
#[cfg(all(feature = "dev", not(target_arch = "wasm32")))]
use bevy_brp_extras::BrpExtrasPlugin;
use bevy_enhanced_input::{
    context::InputContextAppExt,
    prelude::{Action, InputAction, Press as InputPress, Start, actions, bindings},
};
use support::{
    ExampleDriver, FollowCamera, set_camera_preset, spawn_bump_strip, spawn_cargo_truck_demo,
    spawn_compact_car_demo, spawn_drift_coupe_demo, spawn_overlay, spawn_ramp, spawn_rover_demo,
    spawn_skid_vehicle_demo, spawn_world,
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
    pub active: ActiveVehicle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveVehicle {
    Compact,
    Drift,
    Truck,
    Skid,
    Rover,
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

fn main() {
    let mut app = App::new();
    support::configure_example_app(&mut app, "ground_vehicle lab", true);
    app.add_input_context::<LabSwitcher>()
        .add_observer(select_compact)
        .add_observer(select_drift)
        .add_observer(select_truck)
        .add_observer(select_skid)
        .add_observer(select_rover);
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

    commands.insert_resource(LabState {
        compact,
        drift,
        truck,
        skid,
        rover,
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
            )
        ]),
    ));

    spawn_overlay(&mut commands, "ground_vehicle lab");
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
    };
    commands
        .entity(target)
        .insert(bevy_enhanced_input::prelude::ContextActivity::<ExampleDriver>::ACTIVE);
    set_camera_preset(cameras, distance, height, lateral);
}
