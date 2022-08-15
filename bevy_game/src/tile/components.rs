use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_ecs_tilemap_cpu_anim::CPUAnimated;

#[derive(Clone, Copy, Debug, Component, Inspectable)]
pub enum ActivationCondition {
    Odd,
    Even,
}

impl ActivationCondition {
    // TODO don't hardcode the starting side
    pub fn active_on_start(self) -> bool {
        match self {
            ActivationCondition::Odd => false,
            ActivationCondition::Even => true,
        }
    }

    pub(super) fn is_active(self, side: u8) -> bool {
        match self {
            ActivationCondition::Odd => side % 2 == 1,
            ActivationCondition::Even => side % 2 == 0,
        }
    }
}

#[derive(Clone, Copy, Component)]
pub enum ActivatableAnimating {
    // Loops the `on_anim`, without looping the
    // `off_anim`
    Switch {
        on_transition: CPUAnimated,
        off_transition: CPUAnimated,
        on_anim: CPUAnimated,
        off_anim: CPUAnimated,
    },
    // Loops the anim when the tile is on and
    // stops it when off
    Stop,
}

impl ActivatableAnimating {
    pub(in crate::tile) fn update_cpu_anim(&self, anim: &mut CPUAnimated, active: bool) {
        use ActivatableAnimating::*;
        match self {
            Switch { on_anim, off_anim, .. } => if active {
                *anim = *on_anim;
            } else {
                *anim = *off_anim;
            },
            Stop => anim.paused = !active,
        }
    }
}

#[derive(Clone, Copy, Debug, Component, Inspectable)]
pub struct Active { 
    #[inspectable(read_only)]
    pub is_active: bool,
}

#[derive(Component)]
pub struct ConveyorTag;

#[derive(Component)]
pub struct StartTileTag;

#[derive(Component)]
pub struct FrierTag;

#[derive(Component)]
pub struct EndTileTag;
