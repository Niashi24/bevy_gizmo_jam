use bevy::app::App;
use bevy::prelude::*;
use crate::tileset::load::*;

mod load;
mod tile;
mod grid;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<TileGrid>()
            .init_asset_loader::<TileGridAssetLoader>();
    }
}