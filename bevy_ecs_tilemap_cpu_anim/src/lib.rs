use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

use std::time::Duration;

#[derive(StageLabel)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
struct CPUTileAnimateStage;

pub struct CPUTileAnimationPlugin;

impl Plugin for CPUTileAnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(CPUTileAnimations::new())
            .add_stage_before(
                CoreStage::PostUpdate,
                CPUTileAnimateStage,
                SystemStage::parallel()
            )
            .add_system_to_stage(CPUTileAnimateStage, update_animation_frames);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub texture_id: u32, // Texture ID (the bevy_ecs_tilemap one)
    pub duration: Duration, // Duration in milliseconds
}

#[derive(Clone, Debug)]
pub struct CPUTileAnimation(Vec<Frame>);

impl CPUTileAnimation {
    pub fn from_frames(it: impl IntoIterator<Item = Frame>) -> Self {
        CPUTileAnimation(it.into_iter().collect())
    }
}

pub struct CPUTileAnimations(Vec<CPUTileAnimation>);

impl CPUTileAnimations {
    fn new() -> Self { CPUTileAnimations(vec![]) }

    pub fn add_animation(&mut self, anim: CPUTileAnimation) -> usize {
        let id = self.0.len();
        self.0.push(anim);
        id
    }

    pub fn new_cpu_animated(
        &self,
        anim_id: usize,
        looping: bool,
        paused: bool,
    ) -> CPUAnimated {
        CPUAnimated {
            paused, looping, anim_id,
            passed_time: Duration::new(0, 0),
            current_frame: 0,
        }
    }

    pub fn clear(&mut self) { self.0.clear() }
}

#[derive(Clone, Copy, Component, Debug)]
pub struct CPUAnimated {
    pub paused: bool,
    pub looping: bool,
    anim_id: usize,
    current_frame: usize,
    passed_time: Duration,
}

impl CPUAnimated {
    fn update(&mut self, animation: &CPUTileAnimation) -> bool {
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

    fn update_from_anims(&mut self, animations: &CPUTileAnimations) -> bool { 
        self.update(&animations.0[self.anim_id])
    }

    fn tick(&mut self, dt: Duration) {
        if !self.paused {
            self.passed_time += dt;
        }
    }

    pub fn set_animation(
        &mut self, 
        id: usize, 
        paused: bool, 
        looping: bool,
        animations: &CPUTileAnimations
    ) {
        if animations.0.len() <= id { panic!("Bad animation ID"); }
        self.paused = paused;
        self.looping = looping;
        self.anim_id = id;
        self.current_frame = 0;
        self.passed_time = Duration::new(0, 0);
    }
}

pub fn update_animation_frames(
    time: Res<Time>,
    animations: Res<CPUTileAnimations>,
    mut animated_tile_q: Query<(&mut CPUAnimated, &mut TileTexture)>,
) {
    let dt = time.delta();

    animated_tile_q.par_for_each_mut(10, |(mut state, mut tile)| {
        state.tick(dt);
        if state.update_from_anims(&*animations) {
            tile.0 = animations.0[state.anim_id].0[state.current_frame].texture_id;
        }
    });
}
