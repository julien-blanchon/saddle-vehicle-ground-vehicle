use bevy::prelude::*;

#[derive(Message, Debug, Clone, Copy)]
pub struct WheelGroundedChanged {
    pub chassis: Entity,
    pub wheel: Entity,
    pub grounded: bool,
    pub surface_entity: Option<Entity>,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct VehicleBecameAirborne {
    pub chassis: Entity,
}

#[derive(Message, Debug, Clone, Copy)]
pub struct VehicleLanded {
    pub chassis: Entity,
    pub impact_speed_mps: f32,
    pub grounded_wheels: u8,
}
