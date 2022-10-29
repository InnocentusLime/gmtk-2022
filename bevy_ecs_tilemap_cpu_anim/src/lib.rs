use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_ecs_tilemap::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable, InspectableRegistry};

use std::time::Duration;

#[derive(StageLabel)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct CPUTileAnimateStage;

pub struct CPUTileAnimationPlugin;

impl Plugin for CPUTileAnimationPlugin {
    fn build(&self, app: &mut App) {
        if app.world.get_resource::<InspectableRegistry>().is_some() {
            app
                .register_inspectable::<CPUTileAnimation>();
        }

        app
            .add_asset::<CPUTileAnimation>()
            .add_stage_before(
                CoreStage::PostUpdate,
                CPUTileAnimateStage,
                SystemStage::parallel()
            )
            .add_system_to_stage(CPUTileAnimateStage, update_animation_frames);
    }
}

/// An animation frame.
#[derive(Default, Clone, Copy, Debug, Inspectable)]
pub struct Frame {
    /// Texture ID (the bevy_ecs_tilemap one)
    #[inspectable(read_only)]
    pub texture_id: u32,
    /// Duration of the frame
    #[inspectable(read_only)]
    pub duration: Duration,
}

/// The animation asset.
#[derive(Default, Clone, Debug, TypeUuid, Inspectable)]
#[uuid = "deabdd26-c64a-4edb-85d6-f167c53a840a"]
pub struct CPUTileAnimation(#[inspectable(read_only)] Vec<Frame>);

impl CPUTileAnimation {
    pub fn new(it: impl IntoIterator<Item = Frame>) -> Self {
        CPUTileAnimation(it.into_iter().collect())
    }
}

/// The component, that you should attach to the tiles for
/// them to animate. 
/// 
/// # About the animation handle
/// 
/// Currently this plugin mimics bevy's default behaviour for
/// unloaded assets, as in animations that haven't been loaded yet
/// are simply not played and their state is not updated.
/// 
/// # About the default value
/// 
/// The default value of this component is carefully crafted to 
/// not update the texture of the tile it's attached to, while also
/// not referencing any valid assets.
#[derive(Clone, Component, Debug)]
pub struct CPUAnimated {
    pub paused: bool,
    pub looping: bool,
    animation: Handle<CPUTileAnimation>,
    current_frame: usize,
    passed_time: Duration,
}

impl CPUAnimated {
    pub fn new(
        animation: Handle<CPUTileAnimation>,
        looping: bool,
        paused: bool,
    ) -> CPUAnimated {
        CPUAnimated {
            paused, 
            looping, 
            animation,
            passed_time: Duration::new(0, 0),
            current_frame: 0,
        }
    }

    fn update(&mut self, dt: Duration, animation: &CPUTileAnimation) -> bool {
        if self.paused { return false; }

        self.passed_time += dt;

        let old_frame = self.current_frame;
        while self.passed_time > animation.0[self.current_frame].duration {
            if self.current_frame == animation.0.len() - 1 && !self.looping {
                self.passed_time = Duration::new(0, 0);
                break;
            }
            self.passed_time -= animation.0[self.current_frame].duration;
            self.current_frame = (self.current_frame + 1) % animation.0.len();
        }
        old_frame == self.current_frame
    }

    /// Changes the current animation 
    pub fn set_animation(
        &mut self, 
        animation: Handle<CPUTileAnimation>, 
        paused: bool, 
        looping: bool,
    ) {
        self.paused = paused;
        self.looping = looping;
        self.animation = animation;
        self.current_frame = 0;
        self.passed_time = Duration::new(0, 0);
    }
}

impl Default for CPUAnimated {
    fn default() -> Self {
        CPUAnimated {
            paused: false,
            looping: false,
            animation: default(),
            current_frame: 0,
            passed_time: Duration::ZERO,
        }
    }
}

pub fn update_animation_frames(
    time: Res<Time>,
    animations: Res<Assets<CPUTileAnimation>>,
    mut animated_tile_q: Query<(&mut CPUAnimated, &mut TileTexture)>,
) {
    let dt = time.delta();

    animated_tile_q.par_for_each_mut(10, |(mut state, mut tile)| {
        let animation = match animations.get(&state.animation) {
            Some(x) => x,
            None => return,
        };

        if state.update(dt, animation) {
            tile.0 = animation.0[state.current_frame].texture_id;
        }
    });
}
