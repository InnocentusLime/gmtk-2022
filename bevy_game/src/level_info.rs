use bevy::reflect;
use bevy::asset::{ AssetLoader, BoxedFuture, LoadContext, LoadedAsset };

#[derive(serde::Deserialize)]
#[derive(reflect::TypeUuid)]
#[uuid = "9f19fd3b-77fa-4237-9518-95528ecb3f79"]
pub struct LevelInfo {
    level_counts: Vec<u8>,
}

impl LevelInfo {
    pub fn world_count(&self) -> u8 { self.level_counts.len() as u8 }

    pub fn level_count_in_world(&self, world: u8) -> u8 {
        self.level_counts[world as usize]
    }
}

#[derive(Clone, Copy, Default)]
pub struct LevelInfoLoader;

impl AssetLoader for LevelInfoLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            load_context.set_default_asset(LoadedAsset::new(
                serde_json::from_slice::<LevelInfo>(bytes)?
            ));
            Ok(())
        })
    }
    
    fn extensions(&self) -> &[&str] { &["level-info"] }
}
