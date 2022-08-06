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
                bevy_ecs_tilemap::TilemapStage,
                CPUTileAnimateStage,
                SystemStage::parallel()
            )
            .add_system_to_stage(CPUTileAnimateStage, update_animation_frames);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub texture_id: u16, // Texture ID (the bevy_ecs_tilemap one)
    pub duration: Duration, // Duration in milliseconds
}

#[derive(Clone, Debug)]
pub struct CPUTileAnimation(Vec<Frame>);

impl CPUTileAnimation {
    pub fn from_frames(it: impl IntoIterator<Item = Frame>) -> Self {
        CPUTileAnimation(it.into_iter().collect())
    }
}

// TODO Animations -> CPUTileAnimations
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
        speed: f32,
        looping: bool
    ) -> CPUAnimated {
        CPUAnimated {
            speed, looping, anim_id,
            passed_time: Duration::new(0, 0),
            current_frame: 0,
        }
    }

    pub fn clear(&mut self) { self.0.clear() }
}

#[derive(Clone, Copy, Component, Debug)]
pub struct CPUAnimated {
    speed: f32,
    looping: bool,
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
        self.passed_time += dt.mul_f32(self.speed);
    }

    pub fn set_animation(&mut self, id: usize, animations: &CPUTileAnimations) {
        if animations.0.len() <= id { panic!("Bad animation ID"); }
        self.speed = 0.0f32;
        self.anim_id = id;
        self.current_frame = 0;
        self.passed_time = Duration::new(0, 0);
    }

    pub fn set_speed(&mut self, speed: f32) { self.speed = speed }

    pub fn set_looping(&mut self, looping: bool) { self.looping = looping }
}

pub fn update_animation_frames(
    time: Res<Time>,
    animations: Res<CPUTileAnimations>,
    mut animated_tile_q: Query<(&mut CPUAnimated, &mut Tile, &TilePos, &TileParent)>,
    mut map: MapQuery,
) {
    let dt = time.delta();

    // TODO `par_iter_*`
    for (mut state, mut tile, pos, parent_info) in animated_tile_q.iter_mut() {
        state.tick(dt);
        if state.update_from_anims(&*animations) {
            tile.texture_index = animations.0[state.anim_id].0[state.current_frame].texture_id;
            map.notify_chunk_for_tile(*pos, parent_info.map_id, parent_info.layer_id);
        }
    }
}
