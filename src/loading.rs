use crate::state::AppState;
use crate::tileset::load::TileGridAsset;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::Loading)
                .continue_to_state(AppState::Menu)
                .load_collection::<AudioAssets>()
                .load_collection::<TextureAssets>()
                .load_collection::<Levels>(),
        )
            .register_type::<AudioAssets>()
            .register_type::<TextureAssets>()
            .register_type::<Levels>();
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource, Reflect)]
pub struct AudioAssets {
    #[asset(path = "audio/flying.ogg")]
    pub flying: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource, Reflect)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub github: Handle<Image>,
    #[asset(path = "textures/char_temp.png")]
    pub player: Handle<Image>,
    #[asset(path = "textures/block.png")]
    pub block: Handle<Image>,
    #[asset(path = "textures/ramp.png")]
    pub ramp: Handle<Image>,
}

#[derive(AssetCollection, Resource, Reflect)]
pub struct Levels {
    #[asset(path = "levels/level-fall.png")]
    pub test_level: Handle<TileGridAsset>,
    #[asset(
        paths("levels/level-fall.png", "levels/level_fast.png", "levels/test_level.png", ),
        collection(typed, mapped)
    )]
    pub level_map: HashMap<String, Handle<TileGridAsset>>,
}
