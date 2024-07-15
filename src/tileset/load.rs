use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use bevy::asset::{AssetLoader, AsyncReadExt, LoadContext};
use bevy::asset::io::Reader;
use bevy::prelude::*;
use bevy::utils::ConditionalSendFuture;
use image::ImageFormat;
use thiserror::Error;
use crate::tileset::grid::Grid;
use crate::tileset::tile::{Tile, TileImageUnknownPixel};

#[test]
fn load_tilemap() -> io::Result<()> {
    let grid: Grid<Tile> = (&image::io::Reader::open("assets/levels/test_level.png").unwrap()
        .decode().unwrap())
        .try_into().unwrap();

    println!("{}", grid);

    Ok(())
}

#[derive(Asset, TypePath, Debug)]
pub struct TileGridAsset(pub Grid<Tile>);

#[derive(Default)]
pub struct TileGridAssetLoader;

/// Possible errors from loading a TileGridAsset
#[derive(Debug, Error)]
pub enum TileGridAssetLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not load image: {0}")]
    Image(#[from] image::ImageError),
    #[error("Could not parse image: {0}")]
    Pixel(#[from] TileImageUnknownPixel),
}

impl AssetLoader for TileGridAssetLoader {
    type Asset = TileGridAsset;
    type Settings = ();
    type Error = TileGridAssetLoaderError;

    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let img = image::load_from_memory(bytes.as_slice())?;

        let grid = (&img).try_into()?;
        Ok(TileGridAsset(grid))
    }
}

#[derive(Component, Default)]
pub struct TileGridSettings {
    pub solid_texture: Handle<Image>,
    pub ramp_texture: Handle<Image>,
    pub tile_size: f32,
}



#[derive(Bundle, Default)]
pub struct TileGridBundle {
    /// The settings for this TileGrid
    settings: TileGridSettings,
    /// The tile grid being spawned
    tile_grid: Handle<TileGridAsset>,
    /// The local transform of the sprite, relative to its parent.
    pub transform: Transform,
    /// The absolute transform of the sprite. This should generally not be written to directly.
    pub global_transform: GlobalTransform,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Inherited visibility of an entity.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub view_visibility: ViewVisibility,
}

#[derive(Component, Default)]
#[component(storage = "SparseSet")]
struct TileGridLoadingMarker;

fn init_grid(
    query: Query<Entity, Added<TileGridSettings>>,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands.entity(entity)
            .insert(TileGridLoadingMarker);
    }
}

fn spawn_grid(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &TileGridSettings,
            &Handle<TileGridAsset>
        ),
        (
            With<TileGridLoadingMarker>
        )
    >,
    grid_assets: Res<Assets<TileGridAsset>>,
) {
    for (entity, settings, tile_grid) in query.iter() {
        let Some(grid) = grid_assets.get(tile_grid) else {
            continue;
        };

        let mut commands = commands.entity(entity);

        commands.remove::<TileGridLoadingMarker>();

        for ((x, y), tile) in grid.0.iter() {
            let x = x as f32 * settings.tile_size;
            let y = y as f32 * settings.tile_size;
            commands.with_children(|parent| {
                // let texture =

                // parent.spawn(SpriteBundle {
                //     texture:
                //     ..default()
                // });
            });
        }
    }
}