use crate::state::InGame;
use crate::tileset::load::*;
use bevy::app::App;
use bevy::prelude::*;

pub mod grid;
pub mod load;
pub mod tile;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TileGridAsset>()
            .init_asset_loader::<TileGridAssetLoader>()
            .register_type::<TileGridAsset>()
            .add_event::<TileGridLoadEvent>()
            .add_systems(
                Update,
                (spawn_grid, (spawn_colliders, spawn_ramps))
                    .chain()
                    .run_if(in_state(InGame)),
            );
    }
}
