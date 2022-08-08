use bevy_ecs_tilemap_cpu_anim::CPUAnimated;
use bevy_inspector_egui::Inspectable;

#[derive(Clone, Copy, Inspectable)]
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

#[derive(Clone, Copy)]
pub enum ActivatableAnimating {
    // Loops the `on_anim`, without looping the
    // `off_anim`
    //
    // anim_type = switchable_machine
    Switch {
        on_transition: CPUAnimated,
        off_transition: CPUAnimated,
        on_anim: CPUAnimated,
        off_anim: CPUAnimated,
    },
    // Loops the anim when the tile is on and
    // stops it when off
    //
    // anim_type = stoppable_machine
    Stop {
        on_speed: f32,
    },
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
            Stop { on_speed } => if active {
                anim.set_speed(*on_speed);
            } else {
                anim.set_speed(0.0f32);
            },
        }
    }
}

